use std::process::Command;

use super::owned_probe::{self, ProbeSpec};
use crate::services::work_registry::ServiceWorkCancellation;

pub(super) async fn detect_owned(cancel: &ServiceWorkCancellation) -> Option<(u64, u64)> {
    if !cfg!(target_arch = "aarch64") {
        return None;
    }
    let total = owned_probe::run(ProbeSpec::new("sysctl").args(["-n", "hw.memsize"]), cancel);
    let used = owned_probe::run(ProbeSpec::new("vm_stat"), cancel);
    let (total, used) = tokio::join!(total, used);
    let total = parse_total_owned(total?)?;
    let used = parse_used_owned(used?).unwrap_or(0);
    Some((total, used))
}

fn parse_total_owned(output: owned_probe::ProbeOutput) -> Option<u64> {
    if output.truncated {
        return None;
    }
    let raw = String::from_utf8_lossy(&output.stdout);
    Some(raw.trim().parse::<u64>().ok()? / 1_048_576)
}

fn parse_used_owned(output: owned_probe::ProbeOutput) -> Option<u64> {
    if output.truncated {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let page_size = parse_vm_stat_page_size(&text)?;
    let active = parse_vm_stat_field(&text, "Pages active")?;
    let wired = parse_vm_stat_field(&text, "Pages wired down")?;
    let compressed = parse_vm_stat_field(&text, "Pages occupied by compressor").unwrap_or(0);
    let pages = active.saturating_add(wired).saturating_add(compressed);
    Some(pages.saturating_mul(page_size) / 1_048_576)
}

pub(super) fn detect_total() -> Option<u64> {
    if !cfg!(target_arch = "aarch64") {
        return None;
    }
    let output = Command::new("sysctl")
        .args(["-n", "hw.memsize"])
        .output()
        .ok()?;
    let raw = String::from_utf8_lossy(&output.stdout);
    let bytes: u64 = raw.trim().parse().ok()?;
    Some(bytes / 1_048_576)
}

pub(super) fn detect_used() -> Option<u64> {
    if !cfg!(target_arch = "aarch64") {
        return None;
    }
    let output = Command::new("vm_stat").output().ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    let page_size = parse_vm_stat_page_size(&text)?;
    let active = parse_vm_stat_field(&text, "Pages active")?;
    let wired = parse_vm_stat_field(&text, "Pages wired down")?;
    let compressed = parse_vm_stat_field(&text, "Pages occupied by compressor").unwrap_or(0);
    let used_bytes = (active + wired + compressed) * page_size;
    Some(used_bytes / 1_048_576)
}

fn parse_vm_stat_page_size(text: &str) -> Option<u64> {
    let line = text.lines().next()?;
    let num: String = line.chars().filter(|c| c.is_ascii_digit()).collect();
    num.parse().ok()
}

fn parse_vm_stat_field(text: &str, field: &str) -> Option<u64> {
    for line in text.lines() {
        if line.starts_with(field) {
            let val: String = line.chars().filter(|c| c.is_ascii_digit()).collect();
            return val.parse().ok();
        }
    }
    None
}
