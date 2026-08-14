use super::{
    CanonicalDirectory, NativeDirectoryIdentity, OllamaError, StableDirectoryHandle,
    ValidatedPathComponent, VerifiedDirectoryLocation,
};
use std::fs::{self, OpenOptions};
use std::os::windows::fs::{MetadataExt, OpenOptionsExt};
use std::os::windows::io::AsRawHandle;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use windows_sys::Win32::Foundation::{SetHandleInformation, HANDLE, HANDLE_FLAG_INHERIT};
use windows_sys::Win32::Storage::FileSystem::{
    GetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION, FILE_ATTRIBUTE_REPARSE_POINT,
    FILE_FLAG_BACKUP_SEMANTICS,
};

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

fn reject_reparse_components(path: &Path) -> Result<(), OllamaError> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|_| super::OllamaErrorCode::OllamaStorageUnavailable)?
            .join(path)
    };
    let mut current = PathBuf::new();
    for component in absolute.components() {
        current.push(component.as_os_str());
        let metadata = fs::symlink_metadata(&current)
            .map_err(|_| super::OllamaErrorCode::OllamaStorageUnavailable)?;
        if metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            return Err(super::OllamaErrorCode::OllamaModelStoreConflict);
        }
    }
    Ok(())
}

fn opened(path: &Path, _metadata: &fs::Metadata) -> Result<CanonicalDirectory, OllamaError> {
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS)
        .open(path)
        .map_err(|_| super::OllamaErrorCode::OllamaStorageUnavailable)?;
    let raw = file.as_raw_handle();
    let non_inheritable =
        unsafe { SetHandleInformation(raw as HANDLE, HANDLE_FLAG_INHERIT, 0) } != 0;
    if !non_inheritable {
        return Err(super::OllamaErrorCode::OllamaStorageUnavailable);
    }
    let mut info = std::mem::MaybeUninit::<BY_HANDLE_FILE_INFORMATION>::zeroed();
    let success = unsafe { GetFileInformationByHandle(raw as _, info.as_mut_ptr()) } != 0;
    if !success {
        return Err(super::OllamaErrorCode::OllamaStorageUnavailable);
    }
    let info = unsafe { info.assume_init() };
    let file_id = ((info.nFileIndexHigh as u64) << 32) | info.nFileIndexLow as u64;
    Ok(CanonicalDirectory::from_native(
        path.to_path_buf(),
        Some(NativeDirectoryIdentity::windows(
            info.dwVolumeSerialNumber as u64,
            file_id,
        )),
        Some(StableDirectoryHandle(Arc::new(file))),
    ))
}

pub(crate) fn canonical_directory(path: &Path) -> Result<CanonicalDirectory, OllamaError> {
    reject_shape(path)?;
    reject_reparse_components(path)?;
    let canonical =
        dunce::canonicalize(path).map_err(|_| super::OllamaErrorCode::OllamaStorageUnavailable)?;
    let metadata = fs::symlink_metadata(&canonical)
        .map_err(|_| super::OllamaErrorCode::OllamaStorageUnavailable)?;
    if !metadata.is_dir() || metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
        return Err(super::OllamaErrorCode::OllamaModelStoreConflict);
    }
    opened(&canonical, &metadata)
}

pub(crate) fn verified_location(path: &Path) -> Result<VerifiedDirectoryLocation, OllamaError> {
    reject_shape(path)?;
    if let Ok(metadata) = fs::symlink_metadata(path) {
        if !metadata.is_dir() || metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            return Err(super::OllamaErrorCode::OllamaModelStoreConflict);
        }
        reject_reparse_components(path)?;
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

fn normalized(path: &Path) -> String {
    path.to_string_lossy().to_lowercase()
}

pub(crate) fn same_directory(
    left: &CanonicalDirectory,
    right: &CanonicalDirectory,
) -> Result<bool, OllamaError> {
    Ok(match (left.identity(), right.identity()) {
        (Some(left), Some(right)) => left == right,
        _ => normalized(left.path()) == normalized(right.path()),
    })
}

pub(crate) fn contains(
    parent: &CanonicalDirectory,
    child: &CanonicalDirectory,
) -> Result<bool, OllamaError> {
    if same_directory(parent, child)? {
        return Ok(false);
    }
    let parent = normalized(parent.path())
        .trim_end_matches(['/', '\\'])
        .to_owned();
    let child = normalized(child.path());
    Ok(child
        .strip_prefix(&parent)
        .is_some_and(|rest| rest.starts_with(['/', '\\'])))
}
