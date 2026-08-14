#![allow(dead_code)]

use super::constants::{WINDOWS_SHARING_RETRY_INTERVAL, WINDOWS_SHARING_RETRY_TIMEOUT};
use std::fmt;
use std::path::Path;
use std::time::Duration;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum OllamaFsErrorKind {
    NotFound,
    AlreadyExists,
    SharingViolation,
    InvalidInput,
    Other,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct OllamaFsError {
    kind: OllamaFsErrorKind,
    cancelled: bool,
}

impl OllamaFsError {
    pub(super) const fn new(kind: OllamaFsErrorKind) -> Self {
        Self {
            kind,
            cancelled: false,
        }
    }

    fn cancelled() -> Self {
        Self {
            kind: OllamaFsErrorKind::Other,
            cancelled: true,
        }
    }

    pub(super) const fn kind(self) -> OllamaFsErrorKind {
        self.kind
    }

    pub(super) const fn is_cancelled(self) -> bool {
        self.cancelled
    }
}

impl fmt::Display for OllamaFsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("durable filesystem operation failed")
    }
}

impl std::error::Error for OllamaFsError {}

pub(super) trait OllamaDurableFs: Send + Sync {
    fn read_bounded(&self, path: &Path, max_bytes: usize) -> Result<Vec<u8>, OllamaFsError>;
    fn create_directory_durable(&self, path: &Path) -> Result<(), OllamaFsError>;
    fn write_new_atomic(
        &self,
        tmp: &Path,
        final_path: &Path,
        bytes: &[u8],
    ) -> Result<(), OllamaFsError>;
    fn replace_atomic(
        &self,
        tmp: &Path,
        final_path: &Path,
        bytes: &[u8],
    ) -> Result<(), OllamaFsError>;
    fn rename_durable(&self, source: &Path, destination: &Path) -> Result<(), OllamaFsError>;
    fn remove_file_durable(&self, path: &Path) -> Result<(), OllamaFsError>;
    fn remove_tree(&self, root: &Path) -> Result<(), OllamaFsError>;
    fn sync_file(&self, path: &Path) -> Result<(), OllamaFsError>;
    fn sync_parent(&self, path: &Path) -> Result<(), OllamaFsError>;
}

pub(super) fn io_error_kind(error: &std::io::Error) -> OllamaFsErrorKind {
    match error.kind() {
        std::io::ErrorKind::NotFound => OllamaFsErrorKind::NotFound,
        std::io::ErrorKind::AlreadyExists => OllamaFsErrorKind::AlreadyExists,
        std::io::ErrorKind::InvalidInput => OllamaFsErrorKind::InvalidInput,
        _ => OllamaFsErrorKind::Other,
    }
}

pub(super) fn retry_windows_sharing<T, Operation, Cancel, Sleep>(
    mut operation: Operation,
    mut cancelled: Cancel,
    mut sleep: Sleep,
) -> Result<T, OllamaFsError>
where
    Operation: FnMut() -> Result<T, OllamaFsError>,
    Cancel: FnMut() -> bool,
    Sleep: FnMut(Duration),
{
    let max_waits = WINDOWS_SHARING_RETRY_TIMEOUT
        .as_millis()
        .checked_div(WINDOWS_SHARING_RETRY_INTERVAL.as_millis())
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or(0);
    let mut waits = 0;
    loop {
        // L'annulation est testée avant chaque appel OS pour fermer le retry.
        if cancelled() {
            return Err(OllamaFsError::cancelled());
        }
        match operation() {
            Ok(value) => return Ok(value),
            Err(error)
                if error.kind() == OllamaFsErrorKind::SharingViolation && waits < max_waits =>
            {
                sleep(WINDOWS_SHARING_RETRY_INTERVAL);
                waits += 1;
            }
            Err(error) => return Err(error),
        }
    }
}

#[cfg(unix)]
#[path = "durable_fs_unix.rs"]
mod durable_fs_unix;
#[cfg(unix)]
pub(super) use durable_fs_unix::UnixOllamaDurableFs as PlatformOllamaDurableFs;

#[cfg(windows)]
#[path = "durable_fs_windows.rs"]
mod durable_fs_windows;
#[cfg(windows)]
pub(super) use durable_fs_windows::WindowsOllamaDurableFs as PlatformOllamaDurableFs;

pub(super) fn platform_fs() -> PlatformOllamaDurableFs {
    #[cfg(unix)]
    {
        PlatformOllamaDurableFs
    }
    #[cfg(windows)]
    {
        PlatformOllamaDurableFs::default()
    }
}
