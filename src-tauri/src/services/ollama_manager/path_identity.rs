#![allow(dead_code)]

use super::error::OllamaErrorCode;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[cfg(unix)]
#[path = "path_identity_unix.rs"]
pub(crate) mod path_identity_unix;
#[cfg(windows)]
#[path = "path_identity_windows.rs"]
pub(crate) mod path_identity_windows;

pub type OllamaError = OllamaErrorCode;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeDirectoryIdentity {
    value: u128,
}

impl NativeDirectoryIdentity {
    pub(crate) fn synthetic(value: u64) -> Self {
        Self {
            value: value as u128,
        }
    }

    #[cfg(unix)]
    pub(super) fn unix(device: u64, inode: u64) -> Self {
        Self {
            value: ((device as u128) << 64) | inode as u128,
        }
    }

    #[cfg(windows)]
    pub(super) fn windows(volume: u64, file_id: u64) -> Self {
        Self {
            value: ((volume as u128) << 64) | file_id as u128,
        }
    }
}

#[cfg(any(unix, windows))]
#[derive(Clone)]
pub(crate) struct StableDirectoryHandle(pub(crate) Arc<std::fs::File>);

#[derive(Clone)]
pub struct CanonicalDirectory {
    path: PathBuf,
    identity: Option<NativeDirectoryIdentity>,
    #[cfg(any(unix, windows))]
    handle: Option<StableDirectoryHandle>,
}

impl std::fmt::Debug for CanonicalDirectory {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CanonicalDirectory")
            .field("path", &self.path)
            .field("identity", &self.identity)
            .finish()
    }
}

impl PartialEq for CanonicalDirectory {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path && self.identity == other.identity
    }
}

impl Eq for CanonicalDirectory {}

impl CanonicalDirectory {
    pub(crate) fn synthetic(path: PathBuf, identity: Option<NativeDirectoryIdentity>) -> Self {
        #[cfg(any(unix, windows))]
        return Self::from_native(path, identity, None);
        #[cfg(not(any(unix, windows)))]
        Self::from_native(path, identity)
    }

    pub(crate) fn from_native(
        path: PathBuf,
        identity: Option<NativeDirectoryIdentity>,
        #[cfg(any(unix, windows))] handle: Option<StableDirectoryHandle>,
    ) -> Self {
        Self {
            path,
            identity,
            #[cfg(any(unix, windows))]
            handle,
        }
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn identity(&self) -> Option<&NativeDirectoryIdentity> {
        self.identity.as_ref()
    }

    pub(crate) fn child(&self, path: PathBuf, identity: Option<NativeDirectoryIdentity>) -> Self {
        #[cfg(any(unix, windows))]
        return Self::from_native(path, identity, self.handle.clone());
        #[cfg(not(any(unix, windows)))]
        Self::from_native(path, identity)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedPathComponent(OsString);

impl ValidatedPathComponent {
    pub(crate) fn new(value: &str) -> Self {
        Self(OsString::from(value))
    }

    pub(super) fn from_os(value: &std::ffi::OsStr) -> Result<Self, OllamaErrorCode> {
        if value.is_empty()
            || value == std::ffi::OsStr::new(".")
            || value == std::ffi::OsStr::new("..")
            || value.to_string_lossy().contains('\0')
            || value.to_string_lossy().contains(['/', '\\'])
        {
            return Err(OllamaErrorCode::OllamaModelStoreConflict);
        }
        Ok(Self(value.to_owned()))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedDirectoryLocation {
    pub(crate) canonical_parent: CanonicalDirectory,
    pub(crate) leaf: ValidatedPathComponent,
    pub(crate) existing_identity: Option<NativeDirectoryIdentity>,
    pub(crate) existing_directory: Option<CanonicalDirectory>,
}

impl VerifiedDirectoryLocation {
    pub(crate) fn absent(parent: CanonicalDirectory, leaf: ValidatedPathComponent) -> Self {
        Self {
            canonical_parent: parent,
            leaf,
            existing_identity: None,
            existing_directory: None,
        }
    }

    pub(crate) fn existing(directory: CanonicalDirectory) -> Self {
        let identity = directory.identity.clone();
        Self {
            canonical_parent: directory.clone(),
            leaf: ValidatedPathComponent::new(
                directory
                    .path
                    .file_name()
                    .and_then(|part| part.to_str())
                    .unwrap_or("directory"),
            ),
            existing_identity: identity,
            existing_directory: Some(directory),
        }
    }

    pub(super) fn native_existing(
        parent: CanonicalDirectory,
        leaf: ValidatedPathComponent,
        directory: CanonicalDirectory,
    ) -> Self {
        Self {
            canonical_parent: parent,
            leaf,
            existing_identity: directory.identity.clone(),
            existing_directory: Some(directory),
        }
    }

    pub(crate) fn comparison_directory(&self) -> CanonicalDirectory {
        self.existing_directory.clone().unwrap_or_else(|| {
            self.canonical_parent.child(
                self.canonical_parent.path.join(&self.leaf.0),
                self.existing_identity.clone(),
            )
        })
    }
}

pub trait PathIdentityResolver: Send + Sync {
    fn canonical_directory(&self, path: &Path) -> Result<CanonicalDirectory, OllamaError>;
    fn verified_location(&self, path: &Path) -> Result<VerifiedDirectoryLocation, OllamaError>;
    fn same_directory(
        &self,
        left: &CanonicalDirectory,
        right: &CanonicalDirectory,
    ) -> Result<bool, OllamaError>;
    fn contains(
        &self,
        parent: &CanonicalDirectory,
        child: &CanonicalDirectory,
    ) -> Result<bool, OllamaError>;
}

#[allow(unused_imports)]
pub(crate) use super::path_identity_resolver::NativePathIdentityResolver;
