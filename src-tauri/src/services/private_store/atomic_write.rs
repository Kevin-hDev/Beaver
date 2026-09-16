use std::io::Write;
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PublicationDurability {
    Durable,
    PublishedDurabilityUnconfirmed,
}

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
/// Existing callers only need publication. A failure to confirm directory
/// durability is logged because retrying a read-modify-write could apply it twice.
pub(crate) fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let durability = write_with_hook(path, bytes, None, |_| {})?;
    report_unconfirmed(durability);
    Ok(())
}

pub(crate) fn atomic_write_with_durability(
    path: &Path,
    bytes: &[u8],
) -> Result<PublicationDurability, String> {
    write_with_hook(path, bytes, None, |_| {})
}

#[cfg(test)]
pub(super) fn atomic_write_with_hook(
    path: &Path,
    bytes: &[u8],
    hook: impl FnMut(AtomicWriteStage),
) -> Result<(), String> {
    let durability = write_with_hook(path, bytes, None, hook)?;
    report_unconfirmed(durability);
    Ok(())
}

#[cfg(test)]
pub(crate) fn atomic_write_fail_before_replace(path: &Path, bytes: &[u8]) -> Result<(), String> {
    write_with_hook(path, bytes, Some(AtomicWriteStage::FileSynced), |_| {}).map(|_| ())
}

fn write_with_hook(
    path: &Path,
    bytes: &[u8],
    failure: Option<AtomicWriteStage>,
    hook: impl FnMut(AtomicWriteStage),
) -> Result<PublicationDurability, String> {
    write_inner(path, bytes, failure, hook, super::sync_parent)
}

#[cfg(test)]
fn write_with_sync(
    path: &Path,
    bytes: &[u8],
    sync_parent: impl FnOnce(&Path) -> Result<(), String>,
) -> Result<PublicationDurability, String> {
    write_inner(path, bytes, None, |_| {}, sync_parent)
}

fn write_inner(
    path: &Path,
    bytes: &[u8],
    failure: Option<AtomicWriteStage>,
    mut hook: impl FnMut(AtomicWriteStage),
    sync_parent: impl FnOnce(&Path) -> Result<(), String>,
) -> Result<PublicationDurability, String> {
    let parent = path.parent().ok_or_else(super::private_store_error)?;
    super::create_private_dirs(parent)?;
    let temp = super::temp_path(path)?;
    let _temp_cleanup = TempCleanup(&temp);
    let mut file = super::open_private_file(&temp)?;
    hook(AtomicWriteStage::TempOpened);
    file.write_all(bytes)
        .map_err(|_| super::private_store_error())?;
    hook(AtomicWriteStage::ContentWritten);
    file.sync_all().map_err(|_| super::private_store_error())?;
    hook(AtomicWriteStage::FileSynced);
    if failure == Some(AtomicWriteStage::FileSynced) {
        return Err(super::private_store_error());
    }
    super::repair_path(&temp)?;
    hook(AtomicWriteStage::PermissionsRepaired);
    super::replace_file(&temp, path)?;
    hook(AtomicWriteStage::Replaced);
    let durability = match sync_parent(parent) {
        Ok(()) => PublicationDurability::Durable,
        Err(_) => PublicationDurability::PublishedDurabilityUnconfirmed,
    };
    hook(AtomicWriteStage::ParentSynced);
    Ok(durability)
}

fn report_unconfirmed(durability: PublicationDurability) {
    if durability == PublicationDurability::PublishedDurabilityUnconfirmed {
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
    use super::PublicationDurability;

    #[test]
    fn durability_api_distinguishes_a_parent_sync_failure_after_publication() {
        let directory = tempfile::tempdir().expect("directory");
        let path = directory.path().join("state.json");
        let result = super::write_with_sync(&path, b"new", |_| Err("sync failed".into()))
            .expect("published value");

        assert_eq!(
            result,
            PublicationDurability::PublishedDurabilityUnconfirmed
        );
        assert_eq!(std::fs::read(path).unwrap(), b"new");
    }
}
