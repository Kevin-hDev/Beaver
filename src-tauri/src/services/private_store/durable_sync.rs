use std::path::Path;

#[cfg(unix)]
pub(crate) fn sync_directory(path: &Path) -> Result<(), String> {
    std::fs::File::open(path)
        .and_then(|directory| directory.sync_all())
        .map_err(|_| super::private_store_error())
}

#[cfg(not(any(unix, windows)))]
pub(crate) fn sync_directory(_path: &Path) -> Result<(), String> {
    Err(super::private_store_error())
}

#[cfg(windows)]
pub(crate) fn rename_durable(source: &Path, destination: &Path) -> Result<(), String> {
    // Windows confirms the rename itself because directories cannot be flushed as files.
    super::private_store_windows::rename_file(source, destination)
}

#[cfg(unix)]
pub(crate) fn rename_durable(source: &Path, destination: &Path) -> Result<(), String> {
    std::fs::rename(source, destination).map_err(|_| super::private_store_error())?;
    sync_directory(
        destination
            .parent()
            .ok_or_else(super::private_store_error)?,
    )
}

#[cfg(not(any(unix, windows)))]
pub(crate) fn rename_durable(_source: &Path, _destination: &Path) -> Result<(), String> {
    Err(super::private_store_error())
}

pub(super) fn sync_parent(parent: &Path) -> Result<(), String> {
    #[cfg(windows)]
    return Ok(());
    #[cfg(not(windows))]
    sync_directory(parent)
}

pub(super) fn confirm_atomic_publication(parent: &Path) -> Result<(), String> {
    #[cfg(windows)]
    {
        let _ = parent;
        // replace_file uses MOVEFILE_WRITE_THROUGH, which confirms this metadata mutation.
        Ok(())
    }
    #[cfg(not(windows))]
    sync_directory(parent)
}
