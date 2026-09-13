use super::macos::{validate_staged_bundle, ValidatedDestination};
use super::macos_authorization::{AuthorizationSession, ProtectedTool};
use super::macos_install::{ensure_absent, remove_bundle};
use crate::error::InstallerError;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

const MAX_DIRECTORY_ENTRIES: usize = 4_096;
const MAX_ORPHANS: usize = 32;

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum ManagedSiblingKind {
    Stage,
    Backup,
    Failed,
}

pub(super) fn managed_sibling_kind(name: &str) -> Option<ManagedSiblingKind> {
    [
        (".Beaver.app.stage-", ManagedSiblingKind::Stage),
        (".Beaver.app.backup-", ManagedSiblingKind::Backup),
        (".Beaver.app.failed-", ManagedSiblingKind::Failed),
    ]
    .into_iter()
    .find_map(|(prefix, kind)| {
        name.strip_prefix(prefix)
            .is_some_and(|suffix| {
                suffix.len() == 32
                    && suffix
                        .bytes()
                        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            })
            .then_some(kind)
    })
}

struct Orphans {
    backup: Option<PathBuf>,
    disposable: Vec<PathBuf>,
    target_missing: bool,
}

pub fn recover_unprivileged(destination: &ValidatedDestination) -> Result<(), InstallerError> {
    let orphans = inventory(destination)?;
    let target = destination.root().join("Beaver.app");
    if orphans.target_missing {
        if let Some(backup) = &orphans.backup {
            fs::rename(backup, &target).map_err(|_| InstallerError::InstallFailed)?;
            validate_staged_bundle(&target, destination)?;
        }
    }
    for path in removable(&orphans) {
        remove_bundle(path)?;
    }
    Ok(())
}

pub fn recover_authorized(
    destination: &ValidatedDestination,
    session: &mut AuthorizationSession,
) -> Result<(), InstallerError> {
    let orphans = inventory(destination)?;
    let target = destination.root().join("Beaver.app");
    if orphans.target_missing {
        if let Some(backup) = &orphans.backup {
            session.execute(ProtectedTool::Move, &[backup.to_path_buf(), target.clone()])?;
            ensure_absent(backup)?;
            validate_staged_bundle(&target, destination)?;
        }
    }
    for path in removable(&orphans) {
        session.execute(ProtectedTool::Remove, std::slice::from_ref(path))?;
        ensure_absent(path)?;
    }
    Ok(())
}

pub fn remove_authorized(
    path: &Path,
    session: &mut AuthorizationSession,
) -> Result<(), InstallerError> {
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Ok(_) => {}
        Err(_) => return Err(InstallerError::InstallFailed),
    }
    session.execute(
        ProtectedTool::Remove,
        std::slice::from_ref(&path.to_path_buf()),
    )?;
    ensure_absent(path)
}

fn removable(orphans: &Orphans) -> impl Iterator<Item = &PathBuf> {
    orphans.disposable.iter().chain(
        (!orphans.target_missing)
            .then_some(orphans.backup.as_ref())
            .flatten(),
    )
}

fn inventory(destination: &ValidatedDestination) -> Result<Orphans, InstallerError> {
    let target = destination.root().join("Beaver.app");
    let target_missing = match validate_staged_bundle(&target, destination) {
        Ok(_) => false,
        Err(_) => match fs::symlink_metadata(&target) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => true,
            _ => return Err(InstallerError::InstallFailed),
        },
    };
    let mut backup = None;
    let mut disposable = Vec::new();
    for (index, entry) in fs::read_dir(destination.root())
        .map_err(|_| InstallerError::InstallFailed)?
        .enumerate()
    {
        if index >= MAX_DIRECTORY_ENTRIES {
            return Err(InstallerError::InstallFailed);
        }
        let path = entry.map_err(|_| InstallerError::InstallFailed)?.path();
        let Some(name) = path.file_name().and_then(OsStr::to_str) else {
            continue;
        };
        let Some(kind) = managed_sibling_kind(name) else {
            continue;
        };
        validate_staged_bundle(&path, destination)?;
        if disposable.len() + usize::from(backup.is_some()) >= MAX_ORPHANS {
            return Err(InstallerError::InstallFailed);
        }
        if kind == ManagedSiblingKind::Backup {
            if backup.replace(path).is_some() {
                return Err(InstallerError::InstallFailed);
            }
        } else {
            disposable.push(path);
        }
    }
    Ok(Orphans {
        backup,
        disposable,
        target_missing,
    })
}
