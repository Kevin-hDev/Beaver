use std::io::Write;
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum AtomicWriteStage {
    TempOpened,
    ContentWritten,
    FileSynced,
    Replaced,
    PermissionsRepaired,
    ParentSynced,
}

pub(crate) fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    write_with_hook(path, bytes, |_| {})
}

#[cfg(test)]
pub(super) fn atomic_write_with_hook(
    path: &Path,
    bytes: &[u8],
    hook: impl FnMut(AtomicWriteStage),
) -> Result<(), String> {
    write_with_hook(path, bytes, hook)
}

fn write_with_hook(
    path: &Path,
    bytes: &[u8],
    mut hook: impl FnMut(AtomicWriteStage),
) -> Result<(), String> {
    let parent = path.parent().ok_or_else(super::private_store_error)?;
    super::create_private_dirs(parent)?;
    let temp = super::temp_path(path)?;
    let result = (|| {
        let mut file = super::open_private_file(&temp)?;
        hook(AtomicWriteStage::TempOpened);
        file.write_all(bytes)
            .map_err(|_| super::private_store_error())?;
        hook(AtomicWriteStage::ContentWritten);
        file.sync_all().map_err(|_| super::private_store_error())?;
        hook(AtomicWriteStage::FileSynced);
        super::replace_file(&temp, path)?;
        hook(AtomicWriteStage::Replaced);
        super::repair_path(path)?;
        hook(AtomicWriteStage::PermissionsRepaired);
        super::sync_parent(parent)?;
        hook(AtomicWriteStage::ParentSynced);
        Ok(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temp);
    }
    result
}
