use super::owned_probe::{self, ProbeSpec};
use crate::services::work_registry::ServiceWorkCancellation;

pub(super) async fn detect_owned(cancel: &ServiceWorkCancellation) -> Option<(u64, u64)> {
    if let Some(info) = nvidia_smi_info_owned(cancel).await {
        return Some(info);
    }
    if cancel.is_cancelled() {
        return None;
    }
    // sysfs is filesystem I/O even when usually fast, so the blocking pool owns
    // the whole snapshot and no Tokio worker performs a partial scan.
    let snapshot = tokio::task::spawn_blocking(drm_memory_snapshot)
        .await
        .ok()
        .flatten();
    if cancel.is_cancelled() {
        None
    } else {
        snapshot
    }
}

fn drm_memory_snapshot() -> Option<(u64, u64)> {
    let total = drm_memory_mb("mem_info_vram_total", false)
        .or_else(|| drm_memory_mb("mem_info_gtt_total", false));
    let used = drm_memory_mb("mem_info_vram_used", true)
        .filter(|used| *used > 0)
        .or_else(|| drm_memory_mb("mem_info_gtt_used", true));
    (total.is_some() || used.is_some()).then(|| (total.unwrap_or(0), used.unwrap_or(0)))
}

async fn nvidia_smi_info_owned(cancel: &ServiceWorkCancellation) -> Option<(u64, u64)> {
    let output = owned_probe::run(
        ProbeSpec::new("nvidia-smi").args([
            "--query-gpu=memory.total,memory.used",
            "--format=csv,noheader,nounits",
        ]),
        cancel,
    )
    .await?;
    parse_nvidia_rows(&output.stdout, output.truncated)
}

fn parse_nvidia_rows(bytes: &[u8], truncated: bool) -> Option<(u64, u64)> {
    if truncated {
        return None;
    }
    let text = String::from_utf8_lossy(bytes);
    let mut found = false;
    let mut total = 0_u64;
    let mut used = 0_u64;
    for line in text.lines() {
        let mut fields = line.split(',').map(str::trim);
        total = total.saturating_add(fields.next()?.parse::<u64>().ok()?);
        used = used.saturating_add(fields.next()?.parse::<u64>().ok()?);
        found = true;
    }
    found.then_some((total, used))
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
