use super::macos::{validate_bundle, validate_staged_bundle, ValidatedDestination};
use super::macos_authorization::{AuthorizationScope, AuthorizationSession, ProtectedTool};
use super::macos_install::{remove_bundle, swap_authorized, swap_unprivileged, unique_sibling};
use super::macos_recovery::{recover_authorized, recover_unprivileged, remove_authorized};
use crate::error::InstallerError;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum MacInstallResult {
    Installed,
    Reinstalled,
    Cancelled,
}

pub fn install_dmg(
    asset: &Path,
    work_dir: &Path,
    destination: &ValidatedDestination,
    cancellation: &CancellationToken,
    expected_version: &str,
    before_swap: impl FnOnce() -> Result<(), InstallerError>,
) -> Result<MacInstallResult, InstallerError> {
    validate_asset(asset, work_dir)?;
    let mount = match MountedDmg::attach(asset, work_dir, cancellation)? {
        Some(mount) => mount,
        None => return Ok(MacInstallResult::Cancelled),
    };
    let source = validate_bundle(&mount.root.join("Beaver.app"), &mount.root)?;
    let target = destination.root().join("Beaver.app");
    let reinstalled;

    if destination.needs_authorization() {
        let scope = AuthorizationScope::new(destination.root(), &mount.root)?;
        let mut authorization = AuthorizationSession::new(scope)?;
        recover_authorized(destination, &mut authorization)?;
        reinstalled = fs::symlink_metadata(&target).is_ok();
        if cancellation.is_cancelled() {
            return Ok(MacInstallResult::Cancelled);
        }
        let stage = unique_sibling(destination.root(), ".Beaver.app.stage-")?;
        let result = (|| {
            authorization.execute(
                ProtectedTool::Ditto,
                &[source.root().to_path_buf(), stage.clone()],
            )?;
            if cancellation.is_cancelled() {
                remove_authorized(&stage, &mut authorization)?;
                return Ok(true);
            }
            validate_staged_bundle(&stage, destination)?;
            swap_authorized(
                &stage,
                destination,
                &mut authorization,
                expected_version,
                before_swap,
            )?;
            Ok(false)
        })();
        if result.is_err() {
            remove_authorized(&stage, &mut authorization)?;
        }
        if result? {
            return Ok(MacInstallResult::Cancelled);
        }
    } else {
        recover_unprivileged(destination)?;
        reinstalled = fs::symlink_metadata(&target).is_ok();
        let stage = unique_sibling(destination.root(), ".Beaver.app.stage-")?;
        let result = (|| {
            let copied = run_cancellable(
                "/usr/bin/ditto",
                &[source.root().to_path_buf(), stage.clone()],
                cancellation,
            )?;
            if copied == CommandResult::Cancelled {
                remove_bundle(&stage)?;
                return Ok(true);
            }
            validate_staged_bundle(&stage, destination)?;
            if cancellation.is_cancelled() {
                remove_bundle(&stage)?;
                return Ok(true);
            }
            swap_unprivileged(&stage, destination, expected_version, before_swap)?;
            Ok(false)
        })();
        if result.is_err() {
            remove_bundle(&stage)?;
        }
        if result? {
            return Ok(MacInstallResult::Cancelled);
        }
    }

    Ok(if reinstalled {
        MacInstallResult::Reinstalled
    } else {
        MacInstallResult::Installed
    })
}

fn validate_asset(asset: &Path, work_dir: &Path) -> Result<(), InstallerError> {
    let root = work_dir
        .canonicalize()
        .map_err(|_| InstallerError::InstallFailed)?;
    let metadata = fs::symlink_metadata(asset).map_err(|_| InstallerError::InstallFailed)?;
    let canonical = asset
        .canonicalize()
        .map_err(|_| InstallerError::InstallFailed)?;
    if !metadata.is_file()
        || metadata.file_type().is_symlink()
        || canonical.parent() != Some(root.as_path())
        || canonical.extension().and_then(|value| value.to_str()) != Some("dmg")
    {
        return Err(InstallerError::InstallFailed);
    }
    Ok(())
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum CommandResult {
    Completed,
    Cancelled,
}

fn run_cancellable(
    tool: &str,
    arguments: &[PathBuf],
    cancellation: &CancellationToken,
) -> Result<CommandResult, InstallerError> {
    let metadata = fs::symlink_metadata(tool).map_err(|_| InstallerError::InstallFailed)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(InstallerError::InstallFailed);
    }
    let mut child = Command::new(tool)
        .args(arguments)
        .env_clear()
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| InstallerError::InstallFailed)?;
    loop {
        if cancellation.is_cancelled() {
            let _ = child.kill();
            child.wait().map_err(|_| InstallerError::InstallFailed)?;
            return Ok(CommandResult::Cancelled);
        }
        match child
            .try_wait()
            .map_err(|_| InstallerError::InstallFailed)?
        {
            Some(status) if status.success() => return Ok(CommandResult::Completed),
            Some(_) => return Err(InstallerError::InstallFailed),
            None => std::thread::sleep(Duration::from_millis(50)),
        }
    }
}

struct MountedDmg {
    root: PathBuf,
}

impl MountedDmg {
    fn attach(
        asset: &Path,
        work_dir: &Path,
        cancellation: &CancellationToken,
    ) -> Result<Option<Self>, InstallerError> {
        let mut random = [0_u8; 16];
        rand::fill(&mut random);
        let root = work_dir.join(format!("mount-{}", hex::encode(random)));
        let mut builder = fs::DirBuilder::new();
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
        builder
            .create(&root)
            .map_err(|_| InstallerError::InstallFailed)?;
        let arguments = [
            PathBuf::from("attach"),
            PathBuf::from("-readonly"),
            PathBuf::from("-nobrowse"),
            PathBuf::from("-mountpoint"),
            root.clone(),
            asset.to_path_buf(),
        ];
        if run_cancellable("/usr/bin/hdiutil", &arguments, cancellation)?
            == CommandResult::Cancelled
        {
            let _ = fs::remove_dir(&root);
            return Ok(None);
        }
        Ok(Some(Self { root }))
    }
}

impl Drop for MountedDmg {
    fn drop(&mut self) {
        let _ = Command::new("/usr/bin/hdiutil")
            .args(["detach", "-force"])
            .arg(&self.root)
            .env_clear()
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        let _ = fs::remove_dir(&self.root);
    }
}
