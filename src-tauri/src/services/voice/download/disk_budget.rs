use std::path::Path;

use sysinfo::Disks;

const FINALIZATION_MARGIN_BYTES: u64 = 64 * 1024 * 1024;

pub(super) fn required_bytes(archive: u64, installed: u64, existing_partial: u64) -> Option<u64> {
    archive
        .saturating_sub(existing_partial)
        .checked_add(installed)?
        .checked_add(FINALIZATION_MARGIN_BYTES)
}

pub(super) fn missing_bytes(required: u64, available: u64) -> Option<u64> {
    (available < required).then(|| required - available)
}

pub(super) fn shortfall(
    data_dir: &Path,
    archive: u64,
    installed: u64,
    existing_partial: u64,
) -> Result<Option<u64>, String> {
    let required = required_bytes(archive, installed, existing_partial)
        .ok_or_else(|| "model-download-disk-space".to_string())?;
    let disks = Disks::new_with_refreshed_list();
    let available = disks
        .iter()
        .filter(|disk| data_dir.starts_with(disk.mount_point()))
        .max_by_key(|disk| disk.mount_point().as_os_str().len())
        .map(|disk| disk.available_space())
        .ok_or_else(|| "model-download-disk-space".to_string())?;
    Ok(missing_bytes(required, available))
}
