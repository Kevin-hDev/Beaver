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

/// Publishes one complete generation at `path`.
///
/// `Ok(())` means readers can observe the new complete generation. If the
/// directory synchronization fails after publication, the failure is traced
/// but is not returned: retrying a read-modify-write could apply it twice.
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
    let _temp_cleanup = TempCleanup(&temp);
    (|| {
        let mut file = super::open_private_file(&temp)?;
        hook(AtomicWriteStage::TempOpened);
        file.write_all(bytes)
            .map_err(|_| super::private_store_error())?;
        hook(AtomicWriteStage::ContentWritten);
        file.sync_all().map_err(|_| super::private_store_error())?;
        hook(AtomicWriteStage::FileSynced);
        super::repair_path(&temp)?;
        hook(AtomicWriteStage::PermissionsRepaired);
        super::replace_file(&temp, path)?;
        hook(AtomicWriteStage::Replaced);
        report_parent_sync(super::sync_parent(parent));
        hook(AtomicWriteStage::ParentSynced);
        Ok(())
    })()
}

fn report_parent_sync(result: Result<(), String>) {
    if result.is_err() {
        // The destination is already authoritative. Returning a retryable error could
        // repeat a read-modify-write operation against a value that was committed.
        ::log::error!(
            "[private-store] operation=parent-sync result=failed publication=complete durability=unconfirmed"
        );
    }
}

struct TempCleanup<'a>(&'a Path);

impl Drop for TempCleanup<'_> {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(self.0);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn post_publication_sync_failure_is_not_reported_as_a_retryable_write_failure() {
        super::report_parent_sync(Err("sync failed".to_string()));
    }
}
