use super::super::canonical_executable::CanonicalExecutable;
use super::super::durable_fs::MAX_WINDOWS_PATH_UNITS;
use super::{CanonicalDirectory, OllamaError, ValidatedPathComponent, VerifiedDirectoryLocation};
use std::ffi::OsStr;
use std::fs;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::fs::MetadataExt;
use std::path::{Component, Path, PathBuf};
use windows_sys::Win32::Globalization::{CompareStringOrdinal, CSTR_EQUAL};
use windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT;

#[path = "path_identity_windows_handles.rs"]
mod handles;

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

fn reject_reparse_components(path: &Path, allow_missing_final: bool) -> Result<(), OllamaError> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|_| super::OllamaErrorCode::OllamaStorageUnavailable)?
            .join(path)
    };
    let mut current = PathBuf::new();
    let mut components = absolute.components().peekable();
    while let Some(component) = components.next() {
        current.push(component.as_os_str());
        if matches!(component, Component::Prefix(_) | Component::RootDir) {
            continue;
        }
        let is_final = components.peek().is_none();
        let metadata = match fs::symlink_metadata(&current) {
            Ok(metadata) => metadata,
            Err(error)
                if allow_missing_final
                    && is_final
                    && error.kind() == std::io::ErrorKind::NotFound =>
            {
                return Ok(())
            }
            Err(_) => return Err(super::OllamaErrorCode::OllamaStorageUnavailable),
        };
        if metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            return Err(super::OllamaErrorCode::OllamaModelStoreConflict);
        }
    }
    Ok(())
}

pub(crate) fn canonical_directory(path: &Path) -> Result<CanonicalDirectory, OllamaError> {
    reject_shape(path)?;
    reject_reparse_components(path, false)?;
    handles::opened(path)
}

pub(crate) fn verified_location(path: &Path) -> Result<VerifiedDirectoryLocation, OllamaError> {
    reject_shape(path)?;
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if !metadata.is_dir() || metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
            {
                return Err(super::OllamaErrorCode::OllamaModelStoreConflict);
            }
            reject_reparse_components(path, false)?;
            let canonical = canonical_directory(path)?;
            let parent = canonical_directory(
                path.parent()
                    .ok_or(super::OllamaErrorCode::OllamaModelStoreConflict)?,
            )?;
            let leaf = ValidatedPathComponent::from_os(
                path.file_name()
                    .ok_or(super::OllamaErrorCode::OllamaModelStoreConflict)?,
            )?;
            Ok(VerifiedDirectoryLocation::native_existing(
                parent, leaf, canonical,
            ))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            reject_reparse_components(path, true)?;
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
        Err(_) => Err(super::OllamaErrorCode::OllamaStorageUnavailable),
    }
}

pub(crate) fn canonical_executable(path: &Path) -> Result<CanonicalExecutable, OllamaError> {
    reject_shape(path)?;
    reject_reparse_components(path, false)?;
    handles::canonical_executable(path)
}

pub(crate) fn same_directory(
    left: &CanonicalDirectory,
    right: &CanonicalDirectory,
) -> Result<bool, OllamaError> {
    Ok(match (left.identity(), right.identity()) {
        (Some(left), Some(right)) => left == right,
        _ => left.same_unresolved_location(right)?,
    })
}

pub(crate) fn contains(
    parent: &CanonicalDirectory,
    child: &CanonicalDirectory,
) -> Result<bool, OllamaError> {
    if same_directory(parent, child)? {
        return Ok(false);
    }
    if let Some(descendant) = child.unresolved_descendant_of(parent)? {
        return Ok(descendant);
    }
    if let Some(parent_identity) = parent.identity() {
        let ancestors = child.existing_anchor_path().ancestors();
        for ancestor in ancestors {
            if handles::ancestor_identity(ancestor)? == *parent_identity {
                return Ok(true);
            }
        }
        return Ok(false);
    }
    Ok(false)
}

pub(super) fn same_component(left: &OsStr, right: &OsStr) -> Result<bool, OllamaError> {
    let left = bounded_wide(left)?;
    let right = bounded_wide(right)?;
    Ok(unsafe {
        CompareStringOrdinal(
            left.as_ptr(),
            left.len() as i32,
            right.as_ptr(),
            right.len() as i32,
            1,
        ) == CSTR_EQUAL
    })
}

fn bounded_wide(value: &OsStr) -> Result<Vec<u16>, OllamaError> {
    let units = value
        .encode_wide()
        .take(MAX_WINDOWS_PATH_UNITS + 1)
        .collect::<Vec<_>>();
    if units.len() > MAX_WINDOWS_PATH_UNITS || i32::try_from(units.len()).is_err() {
        return Err(super::OllamaErrorCode::OllamaModelStoreConflict);
    }
    Ok(units)
}
