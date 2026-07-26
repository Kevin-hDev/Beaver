use std::process::Command;

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
