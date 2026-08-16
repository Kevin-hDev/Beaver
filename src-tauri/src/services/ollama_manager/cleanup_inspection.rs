#![allow(dead_code)]

use super::constants::MAX_DURABLE_DOCUMENT_BYTES;
use super::durable_fs::{OllamaDurableFs, OllamaFsErrorKind};
use super::error::OllamaErrorCode;
use super::fingerprint::{BundleFingerprint, OllamaVersion};
use super::journal::{classify_migration_marker, OllamaMigrationMarkerClassification};
use super::path_identity::{CanonicalDirectory, NativePathIdentityResolver, PathIdentityResolver};
use super::recovery_decision::{
    ArchiveDirectoryEvidence, DirectoryEvidence, JournalPresence, MigrationMarkerPresence,
    OllamaLayoutSnapshot,
};
use super::spawn_profile_paths::active_executable;
use crate::services::paths::OllamaPaths;
use std::path::Path;

pub(crate) fn marker(fs: &dyn OllamaDurableFs, paths: &OllamaPaths) -> MigrationMarkerPresence {
    match fs.read_bounded(&paths.migration_marker, MAX_DURABLE_DOCUMENT_BYTES) {
        Ok(bytes) => classify_migration_marker(Some(&bytes)).into(),
        Err(error) if error.kind() == OllamaFsErrorKind::NotFound => marker_tmp(paths),
        Err(error) => {
            super::storage_error::record_durable("migration-marker-read", error);
            MigrationMarkerPresence::Invalid
        }
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
        active: evidence(fs, &paths.active),
        install_staging: evidence(fs, &paths.install_staging),
        archive_staging: directory_presence(&paths.archive_staging),
        archive_failed: directory_presence(&paths.archive_failed),
        update_staging: evidence(fs, &paths.update_staging),
        backup: evidence(fs, &paths.backup),
        failed: evidence(fs, &paths.failed),
        legacy_staging: evidence(fs, &paths.legacy_staging),
        legacy_backup: evidence(fs, &paths.legacy_backup),
        backup_delete: deletion_evidence(&paths.backup_delete),
        failed_delete: deletion_evidence(&paths.failed_delete),
    }
}

fn marker_tmp(paths: &OllamaPaths) -> MigrationMarkerPresence {
    match std::fs::symlink_metadata(&paths.migration_marker_tmp) {
        Ok(metadata)
            if metadata.is_file()
                && !metadata.file_type().is_symlink()
                && metadata.len() <= MAX_DURABLE_DOCUMENT_BYTES as u64 =>
        {
            MigrationMarkerPresence::Temporary
        }
        Ok(_) => MigrationMarkerPresence::Invalid,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            MigrationMarkerPresence::Absent
        }
        Err(error) => {
            super::storage_error::record_io("migration-marker-tmp-inspect", &error);
            MigrationMarkerPresence::Unknown
        }
    }
}

fn deletion_evidence(path: &Path) -> DirectoryEvidence {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {
            DirectoryEvidence::Incomplete
        }
        Ok(_) => DirectoryEvidence::Invalid,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => DirectoryEvidence::Absent,
        Err(error) => {
            super::storage_error::record_io("deletion-evidence-inspect", &error);
            DirectoryEvidence::Unknown
        }
    }
}

pub(super) fn directory_presence(path: &Path) -> ArchiveDirectoryEvidence {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {
            ArchiveDirectoryEvidence::Present
        }
        Ok(_) => ArchiveDirectoryEvidence::Invalid,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            ArchiveDirectoryEvidence::Absent
        }
        Err(error) => {
            super::storage_error::record_io("archive-directory-inspect", &error);
            ArchiveDirectoryEvidence::Unknown
        }
    }
}

fn evidence(fs: &dyn OllamaDurableFs, path: &Path) -> DirectoryEvidence {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {
            fingerprint(fs, path).unwrap_or(DirectoryEvidence::Unknown)
        }
        Ok(_) => DirectoryEvidence::Invalid,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => DirectoryEvidence::Absent,
        Err(error) => {
            super::storage_error::record_io("bundle-directory-inspect", &error);
            DirectoryEvidence::Unknown
        }
    }
}

pub(super) fn fingerprint(fs: &dyn OllamaDurableFs, path: &Path) -> Option<DirectoryEvidence> {
    let identity = NativePathIdentityResolver;
    let root = identity
        .canonical_directory(path)
        .map_err(|code| {
            super::storage_error::record_classification("bundle-root-identity", code);
        })
        .ok()?;
    let executable_path = active_executable(root.path());
    let executable = identity
        .canonical_executable(&executable_path)
        .map_err(|code| {
            super::storage_error::record_classification("bundle-executable-identity", code);
        })
        .ok()?;
    let version = fs
        .read_bounded(&root.path().join("VERSION"), 4 * 1024)
        .map_err(|error| {
            super::storage_error::record_durable("bundle-version-read", error);
        })
        .ok()
        .and_then(|bytes| {
            std::str::from_utf8(&bytes)
                .ok()
                .map(str::trim)
                .map(str::to_owned)
        })
        .and_then(|value| OllamaVersion::parse(&value).ok())?;
    let digest = super::probe_http::hash_file(executable.path())
        .map_err(|code| {
            super::storage_error::record_classification(
                "bundle-executable-hash",
                code.diagnostic(),
            );
        })
        .ok()?;
    Some(DirectoryEvidence::Present(BundleFingerprint {
        version,
        executable_sha256: digest,
    }))
}

pub(crate) fn validate_trash(
    path: &Path,
    data_root: &Path,
    models: &CanonicalDirectory,
) -> Result<CanonicalDirectory, OllamaErrorCode> {
    let metadata = std::fs::symlink_metadata(path).map_err(|error| {
        super::storage_error::io(
            "trash-inspect",
            &error,
            OllamaErrorCode::OllamaStorageUnavailable,
        )
    })?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(OllamaErrorCode::OllamaRecoveryDeferred);
    }
    let identity = NativePathIdentityResolver;
    let root = identity.canonical_directory(data_root)?;
    let trash = identity.canonical_directory(path)?;
    if !identity.contains(&root, &trash)? {
        return Err(OllamaErrorCode::OllamaRecoveryDeferred);
    }
    if identity.same_directory(models, &trash)?
        || identity.contains(&trash, models)?
        || identity.contains(models, &trash)?
    {
        return Err(OllamaErrorCode::OllamaModelStoreConflict);
    }
    if trash.identity().is_none() {
        return Err(OllamaErrorCode::OllamaRecoveryDeferred);
    }
    Ok(trash)
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
