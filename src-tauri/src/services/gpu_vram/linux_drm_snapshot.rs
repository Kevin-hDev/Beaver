use std::path::Path;

pub(super) fn read(root: &Path) -> Option<(u64, u64)> {
    let total = memory_mb(root, "mem_info_vram_total", false)
        .or_else(|| memory_mb(root, "mem_info_gtt_total", false));
    let used = memory_mb(root, "mem_info_vram_used", true)
        .filter(|used| *used > 0)
        .or_else(|| memory_mb(root, "mem_info_gtt_used", true));
    (total.is_some() || used.is_some()).then(|| (total.unwrap_or(0), used.unwrap_or(0)))
}

fn memory_mb(root: &Path, file_name: &str, allow_zero: bool) -> Option<u64> {
    let drm = std::fs::read_dir(root).ok()?;
    let mut found = false;
    let mut total = 0_u64;
    for entry in drm.flatten() {
        let path = entry.path().join("device").join(file_name);
        if let Ok(raw) = std::fs::read_to_string(&path) {
            if let Ok(bytes) = raw.trim().parse::<u64>() {
                found = true;
                total = total.saturating_add(bytes);
            }
        }
    }
    if found && (allow_zero || total > 0) {
        Some(total / 1_048_576)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn reads_dedicated_vram_from_a_controlled_sysfs_tree() {
        let root = tempfile::tempdir().expect("sysfs root");
        let device = root.path().join("card0/device");
        std::fs::create_dir_all(&device).expect("device");
        std::fs::write(device.join("mem_info_vram_total"), b"8589934592\n").expect("total");
        std::fs::write(device.join("mem_info_vram_used"), b"2147483648\n").expect("used");

        assert_eq!(super::read(root.path()), Some((8_192, 2_048)));
    }
}
