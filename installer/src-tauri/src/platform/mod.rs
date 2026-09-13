#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "macos")]
pub mod macos_authorization;
#[cfg(target_os = "macos")]
pub mod macos_dmg;
#[cfg(target_os = "macos")]
pub mod macos_install;
pub mod windows;
pub mod windows_cleanup;

use crate::error::InstallerError;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[cfg(target_os = "macos")]
pub fn installed_executable(destination: &Path) -> Result<PathBuf, InstallerError> {
    Ok(macos::installed_bundle(destination)?
        .executable()
        .to_path_buf())
}

#[cfg(target_os = "windows")]
pub fn installed_executable(destination: &Path) -> Result<PathBuf, InstallerError> {
    windows::validate_installed_executable(destination)
}

pub fn launch(executable: &Path) -> Result<(), InstallerError> {
    Command::new(executable)
        .env_clear()
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|_| ())
        .map_err(|_| InstallerError::InstallFailed)
}
