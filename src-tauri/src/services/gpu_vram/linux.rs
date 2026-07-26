use std::process::Command;

pub(super) fn detect_total() -> Option<u64> {
    if let Some(v) = nvidia_smi_vram() {
        return Some(v);
    }
    if let Some(v) = drm_memory_mb("mem_info_vram_total", false) {
        return Some(v);
    }
    if let Some(v) = drm_memory_mb("mem_info_gtt_total", false) {
        return Some(v);
    }
    None
}

pub(super) fn detect_used() -> Option<u64> {
    if let Some(v) = nvidia_smi_field("memory.used") {
        return Some(v);
    }
    if let Some(v) = drm_memory_mb("mem_info_vram_used", true) {
        if v > 0 {
            return Some(v);
        }
    }
    if let Some(v) = drm_memory_mb("mem_info_gtt_used", true) {
        return Some(v);
    }
    None
}

fn drm_memory_mb(file_name: &str, allow_zero: bool) -> Option<u64> {
    let drm = std::fs::read_dir("/sys/class/drm").ok()?;
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

fn nvidia_smi_field(field: &str) -> Option<u64> {
    let output = Command::new("nvidia-smi")
        .args([
            &format!("--query-gpu={field}"),
            "--format=csv,noheader,nounits",
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let raw = String::from_utf8_lossy(&output.stdout);
    raw.lines().next()?.trim().parse::<u64>().ok()
}

fn nvidia_smi_vram() -> Option<u64> {
    nvidia_smi_field("memory.total")
}
