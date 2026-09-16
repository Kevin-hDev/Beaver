use std::path::Path;

#[cfg(unix)]
pub(crate) fn sync_directory(path: &Path) -> Result<(), String> {
    std::fs::File::open(path)
        .and_then(|directory| directory.sync_all())
        .map_err(|_| super::private_store_error())
}

#[cfg(windows)]
pub(crate) fn sync_directory(path: &Path) -> Result<(), String> {
    super::private_store_windows::sync_directory(path)
}

#[cfg(not(any(unix, windows)))]
pub(crate) fn sync_directory(_path: &Path) -> Result<(), String> {
    Err(super::private_store_error())
}

pub(super) fn sync_parent(parent: &Path) -> Result<(), String> {
    sync_directory(parent)
}
