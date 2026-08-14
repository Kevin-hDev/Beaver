use super::super::canonical_executable::CanonicalExecutable;
use super::{CanonicalDirectory, OllamaError, VerifiedDirectoryLocation};
use std::path::{Component, Path};

#[path = "path_identity_unix_handles.rs"]
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

pub(crate) fn canonical_directory(path: &Path) -> Result<CanonicalDirectory, OllamaError> {
    reject_shape(path)?;
    handles::canonical_directory(path)
}

pub(crate) fn verified_location(path: &Path) -> Result<VerifiedDirectoryLocation, OllamaError> {
    reject_shape(path)?;
    handles::verified_location(path)
}

pub(crate) fn canonical_executable(path: &Path) -> Result<CanonicalExecutable, OllamaError> {
    reject_shape(path)?;
    handles::canonical_executable(path)
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
    let Some(parent_identity) = parent.identity() else {
        return Ok(child.path().starts_with(parent.path()));
    };
    let mut ancestors = child.path().ancestors();
    if child.identity().is_none() {
        ancestors.next();
    }
    for ancestor in ancestors {
        if handles::ancestor_identity(ancestor)? == *parent_identity {
            return Ok(true);
        }
    }
    Ok(false)
}
