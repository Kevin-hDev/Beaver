use crate::error::InstallerError;
use std::fs::{File, OpenOptions};
use std::io::{ErrorKind, Read, Write};
use std::path::Path;
use std::time::Duration;

const ACTIVE_MARKER: &str = ".beaver-installer-active";
const LAUNCH_PREFIX: &str = "launch:";
const MAX_PID_BYTES: u64 = 17;
// Covers the launcher-exit/GUI-adoption gap without retaining crashed runs forever.
const LAUNCH_HANDOFF_GRACE: Duration = Duration::from_secs(10 * 60);

pub fn mark(run: &Path) -> Result<(), InstallerError> {
    let path = run.join(ACTIVE_MARKER);
    let mut marker = match OpenOptions::new().write(true).create_new(true).open(&path) {
        Ok(marker) => marker,
        Err(error) if error.kind() == ErrorKind::AlreadyExists => {
            open_existing_without_follow(&path).map_err(|_| InstallerError::CleanupFailed)?
        }
        Err(_) => return Err(InstallerError::CleanupFailed),
    };
    marker
        .set_len(0)
        .map_err(|_| InstallerError::CleanupFailed)?;
    write!(marker, "{}", std::process::id()).map_err(|_| InstallerError::CleanupFailed)?;
    marker.sync_all().map_err(|_| InstallerError::CleanupFailed)
}

pub fn is_active(run: &Path) -> bool {
    let path = run.join(ACTIVE_MARKER);
    let mut marker = match open_without_follow(&path) {
        Ok(marker) => marker,
        Err(error) if error.kind() == ErrorKind::NotFound => return false,
        Err(_) => return true,
    };
    let metadata = match marker.metadata() {
        Ok(metadata) => metadata,
        Err(_) => return true,
    };
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > MAX_PID_BYTES {
        return true;
    }
    let mut value = String::new();
    if marker.read_to_string(&mut value).is_err() {
        return true;
    }
    if let Some(value) = value.strip_prefix(LAUNCH_PREFIX) {
        return valid_pid(value).is_none_or(|pid| {
            process_exists(pid)
                || metadata
                    .modified()
                    .and_then(|created| created.elapsed().map_err(std::io::Error::other))
                    .map(|age| age <= LAUNCH_HANDOFF_GRACE)
                    .unwrap_or(true)
        });
    }
    valid_pid(&value).is_none_or(process_exists)
}

fn valid_pid(value: &str) -> Option<u32> {
    value
        .parse::<u32>()
        .ok()
        .filter(|pid| (2..=i32::MAX as u32).contains(pid))
}

#[cfg(unix)]
fn open_existing_without_follow(path: &Path) -> std::io::Result<File> {
    use std::os::unix::fs::OpenOptionsExt;
    OpenOptions::new()
        .write(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)
}

#[cfg(windows)]
fn open_existing_without_follow(path: &Path) -> std::io::Result<File> {
    use std::os::windows::fs::OpenOptionsExt;
    OpenOptions::new()
        .write(true)
        .custom_flags(0x0020_0000)
        .open(path)
}

#[cfg(unix)]
pub fn open_without_follow(path: &Path) -> std::io::Result<File> {
    use std::os::unix::fs::OpenOptionsExt;
    OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)
}

#[cfg(windows)]
pub fn open_without_follow(path: &Path) -> std::io::Result<File> {
    use std::os::windows::fs::OpenOptionsExt;
    OpenOptions::new()
        .read(true)
        .custom_flags(0x0020_0000)
        .open(path)
}

#[cfg(unix)]
fn process_exists(pid: u32) -> bool {
    let running = unsafe { libc::kill(pid as libc::pid_t, 0) == 0 };
    running || std::io::Error::last_os_error().raw_os_error() == Some(libc::EPERM)
}

#[cfg(windows)]
fn process_exists(pid: u32) -> bool {
    use windows_sys::Win32::Foundation::{
        CloseHandle, GetLastError, ERROR_INVALID_PARAMETER, STILL_ACTIVE,
    };
    use windows_sys::Win32::System::Threading::{
        GetExitCodeProcess, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
    };
    let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
    if handle.is_null() {
        return failed_open_is_active(unsafe { GetLastError() }, ERROR_INVALID_PARAMETER);
    }
    let mut exit_code = 0;
    let queried = unsafe { GetExitCodeProcess(handle, &mut exit_code) } != 0;
    unsafe { CloseHandle(handle) };
    !queried || exit_code == STILL_ACTIVE as u32
}

#[cfg(any(windows, test))]
pub(crate) fn failed_open_is_active(error_code: u32, missing_process_error: u32) -> bool {
    error_code != missing_process_error
}
