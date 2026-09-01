#[cfg(windows)]
use std::ffi::OsStr;
use std::ffi::OsString;
use std::path::{Component, Path, PathBuf};

pub(super) const ROLE_FLAG: &str = "--beaver-terminal-shell-helper";
const MAX_COMMAND_ARGUMENTS: usize = 8;
// Cette borne unique limite l'entrée réservée avant toute création de processus.
const MAX_REQUEST_BYTES: usize = 32 * 1024;
#[cfg(target_os = "linux")]
const HELPER_FAILURE: i32 = 125;

#[cfg_attr(
    not(any(test, target_os = "linux")),
    allow(
        dead_code,
        reason = "the parser stays portable for cross-platform tests but only Linux activates it"
    )
)]
pub(super) struct ShellHelperRequest {
    pub(super) expected_parent: u32,
    pub(super) executable: PathBuf,
    pub(super) arguments: Vec<OsString>,
}

#[cfg_attr(
    not(any(test, target_os = "linux")),
    allow(
        dead_code,
        reason = "the parser stays portable for cross-platform tests but only Linux activates it"
    )
)]
pub(super) fn parse(
    arguments: impl IntoIterator<Item = OsString>,
) -> Result<ShellHelperRequest, ()> {
    let mut arguments = arguments.into_iter();
    let mut total = 0_usize;
    let flag = next_bounded(&mut arguments, &mut total)?;
    if flag != ROLE_FLAG {
        return Err(());
    }
    let parent = next_bounded(&mut arguments, &mut total)?;
    let expected_parent = parent
        .to_str()
        .and_then(|value| value.parse::<u32>().ok())
        .filter(|value| *value > 0)
        .ok_or(())?;
    if next_bounded(&mut arguments, &mut total)? != "--" {
        return Err(());
    }
    let executable = next_bounded(&mut arguments, &mut total)?;
    let executable = canonical_executable(Path::new(&executable))?;
    let mut command_arguments = Vec::with_capacity(MAX_COMMAND_ARGUMENTS);
    for argument in arguments {
        total = total
            .checked_add(argument.as_encoded_bytes().len())
            .filter(|total| *total <= MAX_REQUEST_BYTES)
            .ok_or(())?;
        if command_arguments.len() == MAX_COMMAND_ARGUMENTS {
            return Err(());
        }
        command_arguments.push(argument);
    }
    Ok(ShellHelperRequest {
        expected_parent,
        executable,
        arguments: command_arguments,
    })
}

#[cfg(unix)]
pub(super) fn terminal_shell_executable() -> Result<PathBuf, String> {
    let shell = std::env::var_os("SHELL").unwrap_or_else(|| OsString::from("/bin/bash"));
    canonical_executable(Path::new(&shell)).map_err(|_| "terminal-shell-invalid".to_string())
}

pub(crate) fn run_if_requested() -> Option<i32> {
    #[cfg(not(target_os = "linux"))]
    {
        None
    }
    #[cfg(target_os = "linux")]
    {
        run_linux_if_requested()
    }
}

fn canonical_executable(path: &Path) -> Result<PathBuf, ()> {
    if !path.is_absolute()
        || path
            .components()
            .any(|component| component == Component::ParentDir)
    {
        return Err(());
    }
    let canonical = dunce::canonicalize(path).map_err(|_| ())?;
    if !canonical.is_file() || !is_executable(&canonical)? {
        return Err(());
    }
    Ok(canonical)
}

#[cfg(unix)]
fn is_executable(path: &Path) -> Result<bool, ()> {
    use std::os::unix::fs::PermissionsExt;
    Ok(std::fs::metadata(path)
        .map_err(|_| ())?
        .permissions()
        .mode()
        & 0o111
        != 0)
}

#[cfg(windows)]
fn is_executable(path: &Path) -> Result<bool, ()> {
    let extension = path.extension().and_then(OsStr::to_str).ok_or(())?;
    Ok(["exe", "com"]
        .iter()
        .any(|candidate| extension.eq_ignore_ascii_case(candidate)))
}

fn next_bounded(
    arguments: &mut impl Iterator<Item = OsString>,
    total: &mut usize,
) -> Result<OsString, ()> {
    let value = arguments.next().ok_or(())?;
    *total = total
        .checked_add(value.as_encoded_bytes().len())
        .filter(|total| *total <= MAX_REQUEST_BYTES)
        .ok_or(())?;
    Ok(value)
}

#[cfg(target_os = "linux")]
fn run_linux_if_requested() -> Option<i32> {
    let mut arguments = std::env::args_os().skip(1).peekable();
    if arguments
        .peek()
        .is_none_or(|argument| argument.as_os_str() != std::ffi::OsStr::new(ROLE_FLAG))
    {
        return None;
    }
    let request = match parse(arguments) {
        Ok(request) => request,
        Err(()) => return Some(HELPER_FAILURE),
    };
    if arm_parent_death_signal(request.expected_parent).is_err() {
        return Some(HELPER_FAILURE);
    }
    use std::os::unix::process::CommandExt;
    let error = std::process::Command::new(request.executable)
        .args(request.arguments)
        .exec();
    let _ = error;
    Some(HELPER_FAILURE)
}

#[cfg(target_os = "linux")]
pub(super) fn arm_parent_death_signal(expected_parent: u32) -> Result<(), ()> {
    if expected_parent == 0
        || unsafe { libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL) } != 0
        || unsafe { libc::getppid() } as u32 != expected_parent
    {
        Err(())
    } else {
        Ok(())
    }
}
