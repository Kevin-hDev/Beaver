#![allow(dead_code)]

use super::constants::{WINDOWS_SHARING_RETRY_INTERVAL, WINDOWS_SHARING_RETRY_TIMEOUT};
use super::path_identity::CanonicalDirectory;
use std::path::Path;
use std::time::Duration;

#[path = "durable_fs_error.rs"]
mod durable_fs_error;
#[cfg(any(test, windows))]
pub(super) use durable_fs_error::OllamaFsOperation;
pub(super) use durable_fs_error::{OllamaFsError, OllamaFsErrorKind};

pub(super) const MAX_WINDOWS_PATH_UNITS: usize = 32_768;
pub(super) const WINDOWS_PARENT_FLUSH_ACCESS: u32 = 0x4000_0000;

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
    fn remove_tree_verified(&self, root: &CanonicalDirectory) -> Result<(), OllamaFsError>;
    fn sync_file(&self, path: &Path) -> Result<(), OllamaFsError>;
    fn sync_parent(&self, path: &Path) -> Result<(), OllamaFsError>;
}

pub(super) fn sync_parent_pair<Sync>(
    source: &Path,
    destination: &Path,
    mut sync: Sync,
) -> Result<(), OllamaFsError>
where
    Sync: FnMut(&Path) -> Result<(), OllamaFsError>,
{
    let source_parent = source
        .parent()
        .ok_or_else(|| OllamaFsError::new(OllamaFsErrorKind::InvalidInput))?;
    let destination_parent = destination
        .parent()
        .ok_or_else(|| OllamaFsError::new(OllamaFsErrorKind::InvalidInput))?;
    // Après une publication inter-répertoires, source puis destination ; un seul sync si identiques.
    sync(source_parent)?;
    if source_parent != destination_parent {
        sync(destination_parent)?;
    }
    Ok(())
}

pub(super) fn validate_wide_units<I>(units: I) -> Result<(), OllamaFsErrorKind>
where
    I: IntoIterator<Item = u16>,
{
    let mut count = 0usize;
    for unit in units {
        if unit == 0 {
            return Err(OllamaFsErrorKind::InvalidInput);
        }
        count = count.saturating_add(1);
        if count.saturating_add(1) > MAX_WINDOWS_PATH_UNITS {
            return Err(OllamaFsErrorKind::InvalidInput);
        }
    }
    Ok(())
}

pub(super) const fn windows_file_flush_access() -> u32 {
    WINDOWS_PARENT_FLUSH_ACCESS
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
