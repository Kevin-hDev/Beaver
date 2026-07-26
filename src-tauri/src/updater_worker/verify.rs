use std::ffi::OsStr;
use std::path::{Component, Path, PathBuf};

use subtle::ConstantTimeEq;

use super::{Platform, WorkerError};

#[derive(Debug)]
pub(crate) struct Installation {
    pub(crate) working_directory: PathBuf,
    pub(crate) executable: PathBuf,
    pub(crate) bundle: Option<PathBuf>,
}

pub(crate) fn current_installation(
    working_directory: &Path,
    platform: Platform,
) -> Result<Installation, WorkerError> {
    if !working_directory.is_absolute()
        || working_directory
            .components()
            .any(|component| component == Component::ParentDir)
    {
        return Err(WorkerError);
    }
    let directory_metadata =
        std::fs::symlink_metadata(working_directory).map_err(|_| WorkerError)?;
    if !directory_metadata.file_type().is_dir() || directory_metadata.file_type().is_symlink() {
        return Err(WorkerError);
    }
    let directory = std::fs::canonicalize(working_directory).map_err(|_| WorkerError)?;
    let executable = directory.join(platform.executable_name());
    validate_regular_file(&executable, &directory)?;
    let bundle = if platform == Platform::Macos {
        Some(macos_bundle_from_directory(&directory)?)
    } else {
        None
    };
    Ok(Installation {
        working_directory: directory,
        executable,
        bundle,
    })
}

pub(crate) fn validate_regular_file(path: &Path, root: &Path) -> Result<PathBuf, WorkerError> {
    let metadata = std::fs::symlink_metadata(path).map_err(|_| WorkerError)?;
    if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
        return Err(WorkerError);
    }
    let canonical = std::fs::canonicalize(path).map_err(|_| WorkerError)?;
    if !canonical.starts_with(root) {
        return Err(WorkerError);
    }
    Ok(canonical)
}

fn macos_bundle_from_directory(directory: &Path) -> Result<PathBuf, WorkerError> {
    if directory.file_name() != Some(OsStr::new("MacOS")) {
        return Err(WorkerError);
    }
    let contents = directory.parent().ok_or(WorkerError)?;
    if contents.file_name() != Some(OsStr::new("Contents")) {
        return Err(WorkerError);
    }
    let bundle = contents.parent().ok_or(WorkerError)?;
    match bundle.file_name().and_then(OsStr::to_str) {
        Some("CL-GO.app" | "Beaver.app") => Ok(bundle.to_path_buf()),
        _ => Err(WorkerError),
    }
}

pub(crate) fn valid_health_token(token: &str) -> bool {
    token.len() == 64
        && token
            .as_bytes()
            .iter()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(byte))
}

impl Platform {
    fn executable_name(self) -> &'static str {
        if self == Self::Windows {
            "cl-go-dash.exe"
        } else {
            "cl-go-dash"
        }
    }
}

pub(crate) struct SelfCleanup {
    path: Option<PathBuf>,
}

impl SelfCleanup {
    pub(crate) fn prepare() -> Self {
        Self {
            path: cleanup_path().ok(),
        }
    }
}

fn cleanup_path() -> Result<PathBuf, WorkerError> {
    let executable = std::fs::canonicalize(std::env::current_exe().map_err(|_| WorkerError)?)
        .map_err(|_| WorkerError)?;
    let temp = std::fs::canonicalize(std::env::temp_dir()).map_err(|_| WorkerError)?;
    if !executable.starts_with(temp) {
        return Err(WorkerError);
    }
    let name = executable
        .file_name()
        .and_then(OsStr::to_str)
        .ok_or(WorkerError)?;
    let name = name.strip_suffix(".exe").unwrap_or(name);
    let random = name
        .strip_prefix("cl-go-dash-updater-")
        .ok_or(WorkerError)?;
    if random.len() != 64
        || !random
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(WorkerError);
    }
    Ok(executable)
}

impl Drop for SelfCleanup {
    fn drop(&mut self) {
        let Some(path) = self.path.take() else {
            return;
        };
        cleanup_file(path);
    }
}

#[cfg(not(target_os = "windows"))]
fn cleanup_file(path: PathBuf) {
    let _ = std::fs::remove_file(path);
}

#[cfg(target_os = "windows")]
fn cleanup_file(path: PathBuf) {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{MoveFileExW, MOVEFILE_DELAY_UNTIL_REBOOT};
    let mut wide: Vec<u16> = path.as_os_str().encode_wide().collect();
    wide.push(0);
    unsafe {
        MoveFileExW(wide.as_ptr(), std::ptr::null(), MOVEFILE_DELAY_UNTIL_REBOOT);
    }
}

pub(crate) fn constant_time_token_eq(left: &str, right: &str) -> bool {
    let lengths_match = (left.len() as u64).ct_eq(&(right.len() as u64));
    let mut difference = 0_u8;
    for index in 0..64 {
        let left_byte = left.as_bytes().get(index).copied().unwrap_or_default();
        let right_byte = right.as_bytes().get(index).copied().unwrap_or_default();
        difference |= left_byte ^ right_byte;
    }
    bool::from(lengths_match & difference.ct_eq(&0))
}

#[cfg(test)]
#[path = "verify_tests.rs"]
mod tests;
