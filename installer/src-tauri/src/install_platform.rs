use crate::contract::{InstallerEvent, InstallerOutcome};
use crate::error::InstallerError;
use crate::platform;
use crate::process::executable_is_running;
use crate::runtime::{InstallerRuntime, PlatformKind};
use std::path::{Path, PathBuf};
use tauri::ipc::Channel;
use tokio_util::sync::CancellationToken;

pub fn platform_kind() -> PlatformKind {
    if cfg!(target_os = "macos") {
        PlatformKind::Macos
    } else {
        PlatformKind::Windows
    }
}

#[cfg(target_os = "macos")]
pub fn default_destination() -> Result<PathBuf, InstallerError> {
    Ok(PathBuf::from("/Applications"))
}

#[cfg(target_os = "windows")]
pub fn default_destination() -> Result<PathBuf, InstallerError> {
    let root = std::env::var_os("LOCALAPPDATA").ok_or(InstallerError::InstallFailed)?;
    Ok(PathBuf::from(root).join("Beaver"))
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn default_destination() -> Result<PathBuf, InstallerError> {
    Err(InstallerError::InstallFailed)
}

#[cfg(target_os = "macos")]
pub fn validate_destination(path: &Path) -> Result<(), InstallerError> {
    crate::platform::macos::validate_destination(path).map(|_| ())
}

#[cfg(target_os = "windows")]
pub fn validate_destination(path: &Path) -> Result<(), InstallerError> {
    crate::platform::windows::validate_destination(&path.to_string_lossy()).map(|_| ())
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn validate_destination(_path: &Path) -> Result<(), InstallerError> {
    Err(InstallerError::InstallFailed)
}

pub fn beaver_running(destination: &Path) -> Result<bool, InstallerError> {
    match platform::installed_executable(destination) {
        Ok(executable) => executable_is_running(&executable),
        Err(_) => Ok(false),
    }
}

#[cfg(target_os = "macos")]
pub fn installed_version(destination: &Path) -> Option<String> {
    crate::platform::macos::installed_bundle(destination)
        .and_then(|bundle| crate::platform::macos::bundle_version(&bundle))
        .ok()
}

#[cfg(not(target_os = "macos"))]
pub fn installed_version(_destination: &Path) -> Option<String> {
    None
}

pub fn install_asset(
    asset: &Path,
    run: &Path,
    destination: &Path,
    expected_version: &str,
    operation: &CancellationToken,
    runtime: &InstallerRuntime,
    channel: &Channel<InstallerEvent>,
) -> Result<Option<InstallerOutcome>, InstallerError> {
    #[cfg(target_os = "macos")]
    {
        use crate::platform::macos_dmg::{install_dmg, MacInstallResult};
        let destination = crate::platform::macos::validate_destination(destination)?;
        let _ = channel.send(runtime.installing(true)?);
        let result = install_dmg(
            asset,
            run,
            &destination,
            operation,
            expected_version,
            || {
                channel
                    .send(runtime.begin_swap()?)
                    .map_err(|_| InstallerError::InstallFailed)
            },
        )?;
        let outcome = match result {
            MacInstallResult::Installed => Some(InstallerOutcome::Installed),
            MacInstallResult::Reinstalled => Some(InstallerOutcome::Reinstalled),
            MacInstallResult::Cancelled => None,
        };
        if outcome.is_some() {
            let _ = channel.send(runtime.finishing()?);
        }
        return Ok(outcome);
    }

    #[cfg(target_os = "windows")]
    {
        let _ = expected_version;
        let destination_text = destination.to_string_lossy();
        let validated = crate::platform::windows::validate_destination(&destination_text)?;
        let reinstalled = platform::installed_executable(destination).is_ok();
        let _ = channel.send(runtime.installing(false)?);
        crate::platform::windows::install_nsis(asset, run, &validated, || {
            channel
                .send(runtime.begin_swap()?)
                .map_err(|_| InstallerError::InstallFailed)
        })?;
        let _ = channel.send(runtime.finishing()?);
        return Ok(Some(if reinstalled {
            InstallerOutcome::Reinstalled
        } else {
            InstallerOutcome::Installed
        }));
    }

    #[allow(unreachable_code)]
    Err(InstallerError::InstallFailed)
}
