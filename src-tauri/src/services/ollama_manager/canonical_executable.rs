#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeFileIdentity {
    value: u128,
}

impl NativeFileIdentity {
    pub(crate) fn value(&self) -> u128 {
        self.value
    }

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

    pub(crate) fn value(&self) -> u128 {
        self.identity.value
    }

    pub(crate) fn execution_identity(&self) -> Option<u128> {
        #[cfg(windows)]
        {
            return self.stable_image_identity();
        }
        #[cfg(not(windows))]
        Some(self.value())
    }

    #[cfg(windows)]
    fn stable_image_identity(&self) -> Option<u128> {
        let handle = self.handle.as_ref()?.0.as_ref();
        windows_file_image_identity(handle)
    }

    #[cfg(unix)]
    pub(crate) fn stable_handle(&self) -> Option<&std::fs::File> {
        self.handle.as_ref().map(|handle| handle.0.as_ref())
    }

    #[cfg(unix)]
    pub(crate) fn stable_path_is_current(&self) -> bool {
        use std::os::unix::fs::MetadataExt;

        let Ok(metadata) = std::fs::metadata(&self.path) else {
            return false;
        };
        let value = (u128::from(metadata.dev()) << 64) | u128::from(metadata.ino());
        value == self.identity.value
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

#[cfg(windows)]
pub(crate) fn windows_file_image_identity(file: &std::fs::File) -> Option<u128> {
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::Storage::FileSystem::{
        GetFinalPathNameByHandleW, FILE_NAME_NORMALIZED,
    };
    let mut path = vec![0_u16; 32_768];
    let length = unsafe {
        GetFinalPathNameByHandleW(
            file.as_raw_handle() as _,
            path.as_mut_ptr(),
            path.len() as u32,
            FILE_NAME_NORMALIZED,
        )
    };
    if length == 0 || length as usize >= path.len() {
        return None;
    }
    windows_image_identity_from_path(&path[..length as usize])
}

#[cfg(windows)]
pub(crate) fn windows_image_identity_from_path(path: &[u16]) -> Option<u128> {
    use sha2::{Digest, Sha256};
    let mut value = String::from_utf16(path).ok()?.to_ascii_lowercase();
    if let Some(stripped) = value.strip_prefix(r"\\?\") {
        value = stripped.to_owned();
    }
    let digest = Sha256::digest(value.replace('/', r"\").as_bytes());
    Some(u128::from_be_bytes(digest[..16].try_into().ok()?))
}
