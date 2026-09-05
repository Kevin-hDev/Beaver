use std::path::{Path, PathBuf};

pub(super) fn canonical_file(path: &Path) -> Result<PathBuf, super::OperationFailure> {
    let metadata =
        std::fs::symlink_metadata(path).map_err(|_| super::OperationFailure::RuntimeUnavailable)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(super::OperationFailure::RuntimeUnavailable);
    }
    dunce::canonicalize(path).map_err(|_| super::OperationFailure::RuntimeUnavailable)
}

pub(super) fn canonical_source_file(path: &Path) -> Result<PathBuf, super::OperationFailure> {
    let metadata =
        std::fs::symlink_metadata(path).map_err(|_| super::OperationFailure::ManifestInvalid)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(super::OperationFailure::ManifestInvalid);
    }
    dunce::canonicalize(path).map_err(|_| super::OperationFailure::ManifestInvalid)
}

pub(super) fn canonical_directory(path: &Path) -> Result<PathBuf, super::OperationFailure> {
    let metadata =
        std::fs::symlink_metadata(path).map_err(|_| super::OperationFailure::ManifestInvalid)?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(super::OperationFailure::ManifestInvalid);
    }
    dunce::canonicalize(path).map_err(|_| super::OperationFailure::ManifestInvalid)
}
