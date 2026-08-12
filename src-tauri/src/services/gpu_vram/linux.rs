use std::process::Command;

use super::owned_probe::{self, ProbeSpec};
use crate::services::work_registry::ServiceWorkCancellation;

pub(super) async fn detect_owned(cancel: &ServiceWorkCancellation) -> Option<(u64, u64)> {
    if let Some(info) = nvidia_smi_info_owned(cancel).await {
        return Some(info);
    }
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
