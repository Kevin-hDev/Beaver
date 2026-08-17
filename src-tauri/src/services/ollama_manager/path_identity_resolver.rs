use super::canonical_executable::CanonicalExecutable;
use super::path_identity::{
    CanonicalDirectory, OllamaError, PathIdentityResolver, VerifiedDirectoryLocation,
};
use std::path::Path;

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct NativePathIdentityResolver;

impl PathIdentityResolver for NativePathIdentityResolver {
    fn canonical_directory(&self, path: &Path) -> Result<CanonicalDirectory, OllamaError> {
        platform::canonical_directory(path)
    }

    fn canonical_executable(&self, path: &Path) -> Result<CanonicalExecutable, OllamaError> {
        platform::canonical_executable(path)
    }

    fn verified_location(&self, path: &Path) -> Result<VerifiedDirectoryLocation, OllamaError> {
        platform::verified_location(path)
    }

    fn same_directory(
        &self,
        left: &CanonicalDirectory,
        right: &CanonicalDirectory,
    ) -> Result<bool, OllamaError> {
        platform::same_directory(left, right)
    }

    fn contains(
        &self,
        parent: &CanonicalDirectory,
        child: &CanonicalDirectory,
    ) -> Result<bool, OllamaError> {
        platform::contains(parent, child)
    }
}

#[cfg(unix)]
use super::path_identity::path_identity_unix as platform;
#[cfg(windows)]
use super::path_identity::path_identity_windows as platform;

#[cfg(not(any(unix, windows)))]
mod platform {
    use super::*;

    pub(super) fn canonical_directory(_: &Path) -> Result<CanonicalDirectory, OllamaError> {
        Err(super::super::error::OllamaErrorCode::OllamaStorageUnavailable)
    }
    pub(super) fn verified_location(_: &Path) -> Result<VerifiedDirectoryLocation, OllamaError> {
        Err(super::super::error::OllamaErrorCode::OllamaStorageUnavailable)
    }
    pub(super) fn canonical_executable(_: &Path) -> Result<CanonicalExecutable, OllamaError> {
        Err(super::super::error::OllamaErrorCode::OllamaStorageUnavailable)
    }
    pub(super) fn same_directory(
        _: &CanonicalDirectory,
        _: &CanonicalDirectory,
    ) -> Result<bool, OllamaError> {
        Err(super::super::error::OllamaErrorCode::OllamaStorageUnavailable)
    }
    pub(super) fn contains(
        _: &CanonicalDirectory,
        _: &CanonicalDirectory,
    ) -> Result<bool, OllamaError> {
        Err(super::super::error::OllamaErrorCode::OllamaStorageUnavailable)
    }
}
