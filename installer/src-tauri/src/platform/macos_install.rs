use super::macos::{
    bundle_version, validate_destination, validate_staged_bundle, ValidatedDestination,
};
use super::macos_authorization::{AuthorizationSession, ProtectedTool};
use crate::error::InstallerError;
use std::fs;
use std::path::{Path, PathBuf};

const MAX_NAME_ATTEMPTS: usize = 8;

pub fn swap_unprivileged(
    stage: &Path,
    destination: &ValidatedDestination,
    expected_version: &str,
    before_swap: impl FnOnce() -> Result<(), InstallerError>,
) -> Result<(), InstallerError> {
    revalidate(stage, destination)?;
    verify_expected_bundle(stage, destination, expected_version)?;
    let target = destination.root().join("Beaver.app");
    let backup = existing_backup(&target, destination)?;
    before_swap()?;
    if let Some(backup) = &backup {
        fs::rename(&target, backup).map_err(|_| InstallerError::InstallFailed)?;
    }
    if fs::rename(stage, &target).is_err() {
        if let Some(backup) = &backup {
            let _ = fs::rename(backup, &target);
        }
        return Err(InstallerError::InstallFailed);
    }
    if verify_expected_bundle(&target, destination, expected_version).is_err() {
        rollback_unprivileged(&target, backup.as_deref(), destination)?;
        return Err(InstallerError::InstallFailed);
    }
    if let Some(backup) = backup {
        remove_bundle(&backup)?;
    }
    Ok(())
}

pub fn swap_authorized(
    stage: &Path,
    destination: &ValidatedDestination,
    session: &mut AuthorizationSession,
    expected_version: &str,
    before_swap: impl FnOnce() -> Result<(), InstallerError>,
) -> Result<(), InstallerError> {
    revalidate(stage, destination)?;
    verify_expected_bundle(stage, destination, expected_version)?;
    let target = destination.root().join("Beaver.app");
    let backup = existing_backup(&target, destination)?;
    before_swap()?;
    if let Some(backup) = &backup {
        session.execute(ProtectedTool::Move, &[target.clone(), backup.clone()])?;
        ensure_absent(&target)?;
        validate_staged_bundle(backup, destination)?;
    }
    if session
        .execute(ProtectedTool::Move, &[stage.to_path_buf(), target.clone()])
        .is_err()
    {
        if let Some(backup) = &backup {
            let _ = session.execute(ProtectedTool::Move, &[backup.clone(), target.clone()]);
        }
        return Err(InstallerError::InstallFailed);
    }
    if ensure_absent(stage).is_err()
        || verify_expected_bundle(&target, destination, expected_version).is_err()
    {
        rollback_authorized(&target, backup.as_deref(), session)?;
        return Err(InstallerError::InstallFailed);
    }
    if let Some(backup) = backup {
        session.execute(ProtectedTool::Remove, std::slice::from_ref(&backup))?;
        ensure_absent(&backup)?;
    }
    Ok(())
}

fn revalidate(stage: &Path, destination: &ValidatedDestination) -> Result<(), InstallerError> {
    let current = validate_destination(destination.root())?;
    if current.root() != destination.root()
        || current.needs_authorization() != destination.needs_authorization()
    {
        return Err(InstallerError::InstallFailed);
    }
    validate_staged_bundle(stage, destination)?;
    Ok(())
}

fn verify_expected_bundle(
    path: &Path,
    destination: &ValidatedDestination,
    expected_version: &str,
) -> Result<(), InstallerError> {
    let bundle = validate_staged_bundle(path, destination)?;
    (bundle_version(&bundle)? == expected_version)
        .then_some(())
        .ok_or(InstallerError::InstallFailed)
}

fn ensure_absent(path: &Path) -> Result<(), InstallerError> {
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        _ => Err(InstallerError::InstallFailed),
    }
}

fn existing_backup(
    target: &Path,
    destination: &ValidatedDestination,
) -> Result<Option<PathBuf>, InstallerError> {
    match fs::symlink_metadata(target) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Ok(_) => {
            validate_staged_bundle(target, destination)?;
            Ok(Some(unique_sibling(
                destination.root(),
                ".Beaver.app.backup-",
            )?))
        }
        Err(_) => Err(InstallerError::InstallFailed),
    }
}

fn rollback_unprivileged(
    target: &Path,
    backup: Option<&Path>,
    destination: &ValidatedDestination,
) -> Result<(), InstallerError> {
    let failed = unique_sibling(destination.root(), ".Beaver.app.failed-")?;
    fs::rename(target, &failed).map_err(|_| InstallerError::InstallFailed)?;
    if let Some(backup) = backup {
        fs::rename(backup, target).map_err(|_| InstallerError::InstallFailed)?;
    }
    remove_bundle(&failed)
}

fn rollback_authorized(
    target: &Path,
    backup: Option<&Path>,
    session: &mut AuthorizationSession,
) -> Result<(), InstallerError> {
    let root = target.parent().ok_or(InstallerError::InstallFailed)?;
    let failed = unique_sibling(root, ".Beaver.app.failed-")?;
    session.execute(ProtectedTool::Move, &[target.to_path_buf(), failed.clone()])?;
    ensure_absent(target)?;
    validate_staged_bundle(&failed, &validate_destination(root)?)?;
    if let Some(backup) = backup {
        session.execute(
            ProtectedTool::Move,
            &[backup.to_path_buf(), target.to_path_buf()],
        )?;
        ensure_absent(backup)?;
        validate_staged_bundle(target, &validate_destination(root)?)?;
    }
    session.execute(ProtectedTool::Remove, std::slice::from_ref(&failed))?;
    ensure_absent(&failed)
}

pub fn unique_sibling(root: &Path, prefix: &str) -> Result<PathBuf, InstallerError> {
    for _ in 0..MAX_NAME_ATTEMPTS {
        let mut random = [0_u8; 16];
        rand::fill(&mut random);
        let path = root.join(format!("{prefix}{}", hex::encode(random)));
        match fs::symlink_metadata(&path) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(path),
            Ok(_) => continue,
            Err(_) => return Err(InstallerError::InstallFailed),
        }
    }
    Err(InstallerError::InstallFailed)
}

pub fn remove_bundle(path: &Path) -> Result<(), InstallerError> {
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {
            fs::remove_dir_all(path).map_err(|_| InstallerError::InstallFailed)
        }
        _ => Err(InstallerError::InstallFailed),
    }
}
