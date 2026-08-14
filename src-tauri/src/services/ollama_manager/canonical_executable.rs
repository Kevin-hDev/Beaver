#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeFileIdentity {
    value: u128,
}

impl NativeFileIdentity {
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
pub(crate) struct StableFileHandle(pub(crate) Arc<std::fs::File>);

#[derive(Clone)]
pub struct CanonicalExecutable {
    path: PathBuf,
    identity: NativeFileIdentity,
    #[cfg(any(unix, windows))]
    handle: Option<StableFileHandle>,
}

impl std::fmt::Debug for CanonicalExecutable {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CanonicalExecutable")
            .field("path", &self.path)
            .field("identity", &self.identity)
            .finish()
    }
}

impl PartialEq for CanonicalExecutable {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path && self.identity == other.identity
    }
}

impl Eq for CanonicalExecutable {}

impl CanonicalExecutable {
    pub(crate) fn synthetic(path: PathBuf, identity: NativeFileIdentity) -> Self {
        #[cfg(any(unix, windows))]
        return Self {
            path,
            identity,
            handle: None,
        };
        #[cfg(not(any(unix, windows)))]
        Self { path, identity }
    }

    pub(crate) fn from_native(
        path: PathBuf,
        identity: NativeFileIdentity,
        #[cfg(any(unix, windows))] handle: StableFileHandle,
    ) -> Self {
        Self {
            path,
            identity,
            #[cfg(any(unix, windows))]
            handle: Some(handle),
        }
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    #[allow(dead_code)]
    pub(crate) fn identity(&self) -> &NativeFileIdentity {
        &self.identity
    }

    #[cfg(test)]
    pub(crate) fn has_stable_handle(&self) -> bool {
        #[cfg(any(unix, windows))]
        {
            self.handle.is_some()
        }
        #[cfg(not(any(unix, windows)))]
        {
            false
        }
    }
}
