#![allow(dead_code)]

use super::durable_fs::{OllamaDurableFs, OllamaFsErrorKind};
use super::error::OllamaErrorCode;
use super::journal::{classify_migration_marker, OllamaMigrationMarkerClassification};
use super::path_identity::{NativePathIdentityResolver, PathIdentityResolver};
use super::recovery_decision::{
    DirectoryEvidence, JournalPresence, MigrationMarkerPresence, OllamaLayoutSnapshot,
};
use crate::services::paths::OllamaPaths;
use std::path::{Path, PathBuf};

pub(crate) fn marker(fs: &dyn OllamaDurableFs, paths: &OllamaPaths) -> MigrationMarkerPresence {
    match fs.read_bounded(&paths.migration_marker, 4 * 1024) {
        Ok(bytes) => classify_migration_marker(Some(&bytes)).into(),
        Err(error) if error.kind() == OllamaFsErrorKind::NotFound => {
            MigrationMarkerPresence::Absent
        }
        Err(_) => MigrationMarkerPresence::Invalid,
    }
}

pub(crate) fn snapshot<F: OllamaDurableFs>(
    journal: JournalPresence,
    fs: &F,
    paths: &OllamaPaths,
) -> OllamaLayoutSnapshot {
    OllamaLayoutSnapshot {
        journal,
        migration_marker: marker(fs, paths),
        active: evidence(&paths.active),
        install_staging: evidence(&paths.install_staging),
        update_staging: evidence(&paths.update_staging),
        backup: evidence(&paths.backup),
        failed: evidence(&paths.failed),
        legacy_staging: evidence(&paths.legacy_staging),
        legacy_backup: evidence(&paths.legacy_backup),
        backup_delete: evidence(&paths.backup_delete),
        failed_delete: evidence(&paths.failed_delete),
    }
}

fn evidence(path: &Path) -> DirectoryEvidence {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {
            DirectoryEvidence::Unknown
        }
        Ok(_) => DirectoryEvidence::Invalid,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => DirectoryEvidence::Absent,
        Err(_) => DirectoryEvidence::Unknown,
    }
}

pub(crate) fn validate_trash(path: &Path, data_root: &Path) -> Result<(), OllamaErrorCode> {
    let metadata =
        std::fs::symlink_metadata(path).map_err(|_| OllamaErrorCode::OllamaStorageUnavailable)?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(OllamaErrorCode::OllamaRecoveryDeferred);
    }
    let identity = NativePathIdentityResolver;
    let root = identity.canonical_directory(data_root)?;
    let trash = identity.canonical_directory(path)?;
    if !identity.contains(&root, &trash)? {
        return Err(OllamaErrorCode::OllamaRecoveryDeferred);
    }
    let models = std::env::var_os("OLLAMA_MODELS")
        .map(PathBuf::from)
        .or_else(|| dirs::home_dir().map(|home| home.join(".ollama").join("models")));
    if let Some(models) = models {
        if models.exists() {
            let models = identity.canonical_directory(&models)?;
            if identity.same_directory(&models, &trash)?
                || identity.contains(&trash, &models)?
                || identity.contains(&models, &trash)?
            {
                return Err(OllamaErrorCode::OllamaModelStoreConflict);
            }
        }
    }
    Ok(())
}

impl From<OllamaMigrationMarkerClassification> for MigrationMarkerPresence {
    fn from(value: OllamaMigrationMarkerClassification) -> Self {
        match value {
            OllamaMigrationMarkerClassification::Absent => Self::Absent,
            OllamaMigrationMarkerClassification::Valid(marker) => Self::Valid(marker),
            OllamaMigrationMarkerClassification::Invalid => Self::Invalid,
        }
    }
}
