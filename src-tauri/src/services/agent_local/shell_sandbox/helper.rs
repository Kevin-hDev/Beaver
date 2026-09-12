use std::ffi::OsString;
use std::path::{Component, Path, PathBuf};

const MAX_ARGUMENTS: usize = 64;
const MAX_TOTAL_CHARS: usize = 600_000;
const GUARD_ARG: &str = "--beaver-shell-guard";
const WATCHDOG_ARG: &str = "--beaver-shell-watchdog";

pub(super) fn run_if_requested() -> Option<i32> {
    let mut arguments = std::env::args_os();
    arguments.next()?;
    let role = arguments.next()?;
    if role == super::launch::helper_arg() {
        return Some(exit_code(run(arguments.collect())));
    }
    if role == guard_arg() {
        #[cfg(target_os = "macos")]
        return Some(exit_code(super::macos_parent_guard::run_guarded(
            arguments.collect(),
        )));
        #[cfg(not(target_os = "macos"))]
        return Some(exit_code(Err(super::launch::sandbox_error())));
    }
    if role == watchdog_arg() {
        #[cfg(target_os = "macos")]
        return Some(exit_code(super::macos_parent_guard::run_watchdog(
            arguments.collect(),
        )));
        #[cfg(not(target_os = "macos"))]
        return Some(exit_code(Err(super::launch::sandbox_error())));
    }
    None
}

fn exit_code(result: Result<i32, String>) -> i32 {
    match result {
        Ok(code) => code,
        Err(_) => {
            ::log::error!("Isolation du shell indisponible.");
            126
        }
    }
}

fn run(arguments: Vec<OsString>) -> Result<i32, String> {
    let (mode, temp_dir, executable, command_arguments) = parse(arguments)?;
    validate_temp_dir(&temp_dir)?;
    let roots = super::policy_transport::take(&temp_dir)?;
    let working_dir = dunce::canonicalize(
        std::env::current_dir().map_err(|_| super::launch::sandbox_error())?,
    )
    .map_err(|_| super::launch::sandbox_error())?;
    let scope = match mode {
        super::scope::Mode::Workspace => {
            super::super::directory_access::ensure_allowed_in_roots(&working_dir, &roots)?;
            super::scope::Scope::workspace(roots)
        }
        super::scope::Mode::ProfileCapture => {
            if working_dir != temp_dir {
                return Err(super::launch::sandbox_error());
            }
            super::scope::Scope::profile_capture(roots)
        }
    };

    #[cfg(target_os = "macos")]
    {
        super::macos_parent_guard::install()?;
        return super::macos::run(&executable, &command_arguments, &scope, &temp_dir);
    }
    #[cfg(target_os = "linux")]
    return super::linux::run(&executable, &command_arguments, &scope, &temp_dir);
    #[cfg(windows)]
    return super::windows::run(&executable, &command_arguments, &scope, &temp_dir);
    #[allow(unreachable_code)]
    Err(super::launch::sandbox_error())
}

fn parse(
    arguments: Vec<OsString>,
) -> Result<(super::scope::Mode, PathBuf, PathBuf, Vec<OsString>), String> {
    let (mode, offset) = if arguments.first().and_then(|value| value.to_str())
        == Some(super::launch::profile_capture_arg())
    {
        (super::scope::Mode::ProfileCapture, 1)
    } else {
        (super::scope::Mode::Workspace, 0)
    };
    if arguments.len() < offset + 3 || arguments.len() > MAX_ARGUMENTS + offset + 3 {
        return Err(super::launch::sandbox_error());
    }
    validate_total_chars(&arguments)?;
    if arguments.get(offset + 1).and_then(|value| value.to_str()) != Some("--") {
        return Err(super::launch::sandbox_error());
    }
    let temp_dir = PathBuf::from(super::launch::os_text(&arguments[offset])?);
    let executable = PathBuf::from(super::launch::os_text(&arguments[offset + 2])?);
    if !valid_path_shape(&temp_dir) || !valid_path_shape(&executable) {
        return Err(super::launch::sandbox_error());
    }
    let executable = dunce::canonicalize(executable).map_err(|_| super::launch::sandbox_error())?;
    if !executable.is_file() {
        return Err(super::launch::sandbox_error());
    }
    let command_arguments = arguments[offset + 3..]
        .iter()
        .map(|value| super::launch::os_text(value))
        .collect::<Result<Vec<_>, _>>()?;
    Ok((mode, temp_dir, executable, command_arguments))
}

pub(super) fn parse_guarded_command(
    arguments: Vec<OsString>,
) -> Result<(PathBuf, Vec<OsString>), String> {
    if arguments.len() < 2 || arguments.len() > MAX_ARGUMENTS + 2 {
        return Err(super::launch::sandbox_error());
    }
    validate_total_chars(&arguments)?;
    if arguments.first().and_then(|value| value.to_str()) != Some("--") {
        return Err(super::launch::sandbox_error());
    }
    let executable = PathBuf::from(super::launch::os_text(&arguments[1])?);
    if !valid_path_shape(&executable) {
        return Err(super::launch::sandbox_error());
    }
    let executable = dunce::canonicalize(executable).map_err(|_| super::launch::sandbox_error())?;
    if !executable.is_file() {
        return Err(super::launch::sandbox_error());
    }
    let command_arguments = arguments[2..]
        .iter()
        .map(|value| super::launch::os_text(value))
        .collect::<Result<Vec<_>, _>>()?;
    Ok((executable, command_arguments))
}

fn validate_total_chars(arguments: &[OsString]) -> Result<(), String> {
    let mut total = 0_usize;
    for value in arguments {
        total = total
            .checked_add(value.to_string_lossy().chars().count())
            .ok_or_else(super::launch::sandbox_error)?;
        if total > MAX_TOTAL_CHARS {
            return Err(super::launch::sandbox_error());
        }
    }
    Ok(())
}

pub(super) fn guard_arg() -> &'static str {
    GUARD_ARG
}

pub(super) fn watchdog_arg() -> &'static str {
    WATCHDOG_ARG
}

fn valid_path_shape(path: &Path) -> bool {
    path.is_absolute()
        && path.as_os_str().to_string_lossy().chars().count()
            <= super::super::directory_access::MAX_PATH_CHARS
        && !path
            .components()
            .any(|part| matches!(part, Component::ParentDir))
}

fn validate_temp_dir(path: &Path) -> Result<(), String> {
    let root = super::launch::sandbox_temp_root();
    let canonical_root = dunce::canonicalize(root).map_err(|_| super::launch::sandbox_error())?;
    let canonical = dunce::canonicalize(path).map_err(|_| super::launch::sandbox_error())?;
    let direct_child = canonical.parent() == Some(canonical_root.as_path());
    if !direct_child || !canonical.is_dir() {
        return Err(super::launch::sandbox_error());
    }
    Ok(())
}

#[cfg(test)]
#[path = "helper_tests.rs"]
mod tests;
