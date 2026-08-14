use super::{
    CanonicalDirectory, NativeDirectoryIdentity, OllamaError, StableDirectoryHandle,
    ValidatedPathComponent, VerifiedDirectoryLocation,
};
use std::fs::{self, File};
use std::os::unix::fs::MetadataExt;
use std::path::{Component, Path};
use std::sync::Arc;

fn reject_shape(path: &Path) -> Result<(), OllamaError> {
    if path.as_os_str().is_empty()
        || path.as_os_str().to_string_lossy().contains('\0')
        || path
            .components()
            .any(|part| matches!(part, Component::CurDir | Component::ParentDir))
    {
        return Err(super::OllamaErrorCode::OllamaModelStoreConflict);
    }
    Ok(())
}

fn reject_symlink_components(path: &Path) -> Result<(), OllamaError> {
    if fs::symlink_metadata(path)
        .map_err(|_| super::OllamaErrorCode::OllamaStorageUnavailable)?
        .file_type()
        .is_symlink()
    {
        return Err(super::OllamaErrorCode::OllamaModelStoreConflict);
    }
    Ok(())
}

fn opened(path: &Path, metadata: &fs::Metadata) -> Result<CanonicalDirectory, OllamaError> {
    let file = File::open(path).map_err(|_| super::OllamaErrorCode::OllamaStorageUnavailable)?;
    Ok(CanonicalDirectory::from_native(
        path.to_path_buf(),
        Some(NativeDirectoryIdentity::unix(
            metadata.dev(),
            metadata.ino(),
        )),
        Some(StableDirectoryHandle(Arc::new(file))),
    ))
}

pub(crate) fn canonical_directory(path: &Path) -> Result<CanonicalDirectory, OllamaError> {
    reject_shape(path)?;
    reject_symlink_components(path)?;
    let canonical =
        fs::canonicalize(path).map_err(|_| super::OllamaErrorCode::OllamaStorageUnavailable)?;
    let metadata = fs::symlink_metadata(&canonical)
        .map_err(|_| super::OllamaErrorCode::OllamaStorageUnavailable)?;
    if !metadata.is_dir() {
        return Err(super::OllamaErrorCode::OllamaStorageUnavailable);
    }
    opened(&canonical, &metadata)
}

pub(crate) fn verified_location(path: &Path) -> Result<VerifiedDirectoryLocation, OllamaError> {
    reject_shape(path)?;
    if let Ok(metadata) = fs::symlink_metadata(path) {
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(super::OllamaErrorCode::OllamaModelStoreConflict);
        }
        reject_symlink_components(path)?;
        let canonical = canonical_directory(path)?;
        let parent = canonical_directory(
            path.parent()
                .ok_or(super::OllamaErrorCode::OllamaModelStoreConflict)?,
        )?;
        let leaf = ValidatedPathComponent::from_os(
            path.file_name()
                .ok_or(super::OllamaErrorCode::OllamaModelStoreConflict)?,
        )?;
        return Ok(VerifiedDirectoryLocation::native_existing(
            parent, leaf, canonical,
        ));
    }
    let parent = canonical_directory(
        path.parent()
            .ok_or(super::OllamaErrorCode::OllamaModelStoreConflict)?,
    )?;
    let leaf = ValidatedPathComponent::from_os(
        path.file_name()
            .ok_or(super::OllamaErrorCode::OllamaModelStoreConflict)?,
    )?;
    Ok(VerifiedDirectoryLocation::absent(parent, leaf))
}

pub(crate) fn same_directory(
    left: &CanonicalDirectory,
    right: &CanonicalDirectory,
) -> Result<bool, OllamaError> {
    Ok(match (left.identity(), right.identity()) {
        (Some(left), Some(right)) => left == right,
        _ => left.path() == right.path(),
    })
}

pub(crate) fn contains(
    parent: &CanonicalDirectory,
    child: &CanonicalDirectory,
) -> Result<bool, OllamaError> {
    if same_directory(parent, child)? {
        return Ok(false);
    }
    Ok(child.path().starts_with(parent.path()))
}
