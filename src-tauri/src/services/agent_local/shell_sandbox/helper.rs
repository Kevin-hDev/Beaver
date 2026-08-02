use std::ffi::OsString;
use std::path::{Component, Path, PathBuf};

const MAX_ARGUMENTS: usize = 64;
const MAX_TOTAL_CHARS: usize = 600_000;

pub(super) fn run_if_requested() -> Option<i32> {
    let mut arguments = std::env::args_os();
    arguments.next()?;
    if arguments.next().as_deref() != Some(std::ffi::OsStr::new(super::launch::helper_arg())) {
        return None;
    }
    Some(match run(arguments.collect()) {
        Ok(code) => code,
        Err(_) => {
            eprintln!("Isolation du shell indisponible.");
            126
        }
    })
}

fn run(arguments: Vec<OsString>) -> Result<i32, String> {
    let (temp_dir, executable, command_arguments) = parse(arguments)?;
    let roots = super::super::directory_access::configured_roots()?;
    let working_dir = dunce::canonicalize(
        std::env::current_dir().map_err(|_| super::launch::sandbox_error())?,
    )
    .map_err(|_| super::launch::sandbox_error())?;
    super::super::directory_access::ensure_allowed_in_roots(&working_dir, &roots)?;
    validate_temp_dir(&temp_dir)?;

    #[cfg(target_os = "macos")]
    return super::macos::run(&executable, &command_arguments, &roots, &temp_dir);
    #[cfg(target_os = "linux")]
    return super::linux::run(&executable, &command_arguments, &roots, &temp_dir);
    #[cfg(windows)]
    return super::windows::run(&executable, &command_arguments, &roots, &temp_dir);
    #[allow(unreachable_code)]
    Err(super::launch::sandbox_error())
}

fn parse(arguments: Vec<OsString>) -> Result<(PathBuf, PathBuf, Vec<OsString>), String> {
    if arguments.len() < 4 || arguments.len() > MAX_ARGUMENTS + 3 {
        return Err(super::launch::sandbox_error());
    }
    let mut total = 0_usize;
    for value in &arguments {
        total = total
            .checked_add(value.to_string_lossy().chars().count())
            .ok_or_else(super::launch::sandbox_error)?;
        if total > MAX_TOTAL_CHARS {
            return Err(super::launch::sandbox_error());
        }
    }
    if arguments.get(1).and_then(|value| value.to_str()) != Some("--") {
        return Err(super::launch::sandbox_error());
    }
    let temp_dir = PathBuf::from(super::launch::os_text(&arguments[0])?);
    let executable = PathBuf::from(super::launch::os_text(&arguments[2])?);
    if !valid_path_shape(&temp_dir) || !valid_path_shape(&executable) {
        return Err(super::launch::sandbox_error());
    }
    let executable = dunce::canonicalize(executable).map_err(|_| super::launch::sandbox_error())?;
    if !executable.is_file() {
        return Err(super::launch::sandbox_error());
    }
    let command_arguments = arguments[3..]
        .iter()
        .map(|value| super::launch::os_text(value))
        .collect::<Result<Vec<_>, _>>()?;
    Ok((temp_dir, executable, command_arguments))
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
