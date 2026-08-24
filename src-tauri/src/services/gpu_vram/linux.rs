use super::owned_probe::{self, ProbeSpec};
use crate::services::work_registry::ServiceWorkCancellation;

pub(super) async fn detect_owned(cancel: &ServiceWorkCancellation) -> Option<(u64, Option<u64>)> {
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
        snapshot.map(|(total_mb, used_mb)| (total_mb, Some(used_mb)))
    }
}

fn drm_memory_snapshot() -> Option<(u64, u64)> {
    super::linux_drm_snapshot::read(std::path::Path::new("/sys/class/drm"))
}

async fn nvidia_smi_info_owned(cancel: &ServiceWorkCancellation) -> Option<(u64, Option<u64>)> {
    let output = owned_probe::run(
        ProbeSpec::new("nvidia-smi").args([
            "--query-gpu=memory.total,memory.used",
            "--format=csv,noheader,nounits",
        ]),
        cancel,
    )
    .await?;
    parse_nvidia_rows(&output.stdout, output.truncated)
        .map(|(total_mb, used_mb)| (total_mb, Some(used_mb)))
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
