use super::error::OllamaErrorCode;
use super::path_identity::{CanonicalDirectory, PathIdentityResolver};
use super::spawn_environment::FrozenEnvironment;
use crate::services::paths::OllamaPaths;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::path::{Component, Path, PathBuf};

pub(crate) fn resolve_models_path(
    raw: Option<&OsStr>,
    cwd: &CanonicalDirectory,
    environment: &FrozenEnvironment,
) -> Result<PathBuf, OllamaErrorCode> {
    let path = match raw {
        Some(value) if value.is_empty() => return Err(OllamaErrorCode::OllamaModelStoreConflict),
        Some(value) => PathBuf::from(value),
        None => {
            let home = environment
                .value("HOME")
                .or_else(|| environment.value("USERPROFILE"))
                .ok_or(OllamaErrorCode::OllamaStorageUnavailable)?;
            let home = PathBuf::from(home);
            if !home.is_absolute()
                || home.components().any(|part| {
                    matches!(
                        part,
                        std::path::Component::CurDir | std::path::Component::ParentDir
                    )
                })
            {
                return Err(OllamaErrorCode::OllamaStorageUnavailable);
            }
            home.join(".ollama").join("models")
        }
    };
    if path.as_os_str().is_empty()
        || path
            .components()
            .any(|part| matches!(part, Component::CurDir | Component::ParentDir))
    {
        return Err(OllamaErrorCode::OllamaModelStoreConflict);
    }
    #[cfg(windows)]
    if path
        .components()
        .next()
        .is_some_and(|part| matches!(part, Component::Prefix(_)))
        && !path.has_root()
    {
        return Err(OllamaErrorCode::OllamaModelStoreConflict);
    }
    Ok(if path.is_absolute() {
        path
    } else {
        cwd.path().join(path)
    })
}

pub(crate) fn verified_models_directory(
    path: &Path,
    identity: &dyn PathIdentityResolver,
) -> Result<CanonicalDirectory, OllamaErrorCode> {
    match identity.verified_location(path) {
        Ok(location) => return Ok(location.comparison_directory()),
        Err(OllamaErrorCode::OllamaStorageUnavailable) => {}
        Err(code) => return Err(code),
    }
    let mut cursor = path;
    let mut missing = Vec::<OsString>::new();
    let existing = loop {
        match std::fs::symlink_metadata(cursor) {
            Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {
                break identity.canonical_directory(cursor)?;
            }
            Ok(_) => return Err(OllamaErrorCode::OllamaModelStoreConflict),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                if missing.len() >= super::constants::MAX_OLLAMA_PATH_COMPONENTS {
                    return Err(OllamaErrorCode::OllamaModelStoreConflict);
                }
                missing.push(
                    cursor
                        .file_name()
                        .ok_or(OllamaErrorCode::OllamaModelStoreConflict)?
                        .to_owned(),
                );
                cursor = cursor
                    .parent()
                    .ok_or(OllamaErrorCode::OllamaModelStoreConflict)?;
            }
            Err(_) => return Err(OllamaErrorCode::OllamaStorageUnavailable),
        }
    };
    missing
        .into_iter()
        .rev()
        .try_fold(existing, |parent, name| {
            Ok(
                parent.unresolved_child(super::path_identity::ValidatedPathComponent::from_os(
                    &name,
                )?),
            )
        })
}

pub(super) fn active_executable(active: &Path) -> PathBuf {
    let name = if cfg!(windows) {
        "ollama.exe"
    } else {
        "ollama"
    };
    let modern = active.join("bin").join(name);
    match std::fs::symlink_metadata(&modern) {
        Ok(_) => modern,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            // Beaver 1.1.x extrayait le binaire publié à la racine du bundle.
            // Le layout moderne reste prioritaire dès qu'une entrée y existe.
            active.join(name)
        }
        Err(_) => modern,
    }
}

pub(super) fn transaction_locations(paths: &OllamaPaths, probe: bool) -> Vec<&PathBuf> {
    let all = [
        &paths.active,
        &paths.legacy_staging,
        &paths.legacy_backup,
        &paths.failed,
        &paths.install_staging,
        &paths.archive_staging,
        &paths.archive_failed,
        &paths.uncommitted_staging_delete,
        &paths.update_staging,
        &paths.backup,
        &paths.backup_delete,
        &paths.failed_delete,
        &paths.probe_models,
    ];
    all.into_iter()
        .filter(|path| !(probe && *path == &paths.probe_models))
        .collect()
}

pub(super) fn overlaps(
    identity: &dyn PathIdentityResolver,
    models: &CanonicalDirectory,
    transaction: &CanonicalDirectory,
) -> Result<bool, OllamaErrorCode> {
    let equal = identity.same_directory(models, transaction)?;
    let models_parent = identity.contains(models, transaction)?;
    let transaction_parent = identity.contains(transaction, models)?;
    Ok(equal || models_parent || transaction_parent)
}
