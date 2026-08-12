#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
mod owned_probe;
#[cfg(test)]
mod owned_probe_tests;
mod snapshot;
#[cfg(target_os = "windows")]
mod windows;

use snapshot::SnapshotCache;
use std::sync::LazyLock;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GpuVramSnapshot {
    pub total_mb: u64,
    pub used_mb: u64,
}

static SNAPSHOT: LazyLock<SnapshotCache> = LazyLock::new(SnapshotCache::default);

const VRAM_TIER_HIGH_MB: u64 = 24_000;
const VRAM_TIER_MID_MB: u64 = 12_000;
const CTX_HIGH: u32 = 32768;
const CTX_MID: u32 = 24576;
const CTX_LOW: u32 = 8192;

pub fn detect_vram_mb() -> Option<u64> {
    SNAPSHOT.get().map(|snapshot| snapshot.total_mb)
}

pub fn detect_vram_used_mb() -> Option<u64> {
    SNAPSHOT.get().map(|snapshot| snapshot.used_mb)
}

#[allow(clippy::needless_return)] // pattern multi-cfg cross-plateforme
pub async fn refresh_owned(
    cancel: crate::services::work_registry::ServiceWorkCancellation,
) -> Option<GpuVramSnapshot> {
    let measurement = detect_owned(&cancel)
        .await
        .map(|(total_mb, used_mb)| GpuVramSnapshot { total_mb, used_mb });
    // La sonde possédée est l'unique auteur. Un échec efface l'ancienne
    // mesure afin qu'une valeur obsolète ne permette pas un modèle trop gros.
    SNAPSHOT.replace(measurement);
    measurement
}

#[allow(clippy::needless_return)] // pattern multi-cfg cross-plateforme
async fn detect_owned(
    cancel: &crate::services::work_registry::ServiceWorkCancellation,
) -> Option<(u64, u64)> {
    #[cfg(target_os = "macos")]
    {
        macos::detect_owned(cancel).await
    }
    #[cfg(target_os = "linux")]
    {
        return linux::detect_owned(cancel).await;
    }
    #[cfg(target_os = "windows")]
    {
        return windows::detect_owned(cancel).await;
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    None
}

pub fn compute_default_num_ctx() -> u32 {
    num_ctx_for_vram(detect_vram_mb())
}

fn num_ctx_for_vram(vram_mb: Option<u64>) -> u32 {
    match vram_mb {
        Some(mb) if mb >= VRAM_TIER_HIGH_MB => CTX_HIGH,
        Some(mb) if mb >= VRAM_TIER_MID_MB => CTX_MID,
        _ => CTX_LOW,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn published_snapshot_is_the_only_synchronous_measurement_source() {
        SNAPSHOT.replace(Some(GpuVramSnapshot {
            total_mb: 24_576,
            used_mb: 8_192,
        }));

        assert_eq!(detect_vram_mb(), Some(24_576));
        assert_eq!(detect_vram_used_mb(), Some(8_192));
        assert_eq!(compute_default_num_ctx(), CTX_HIGH);
        SNAPSHOT.replace(None);
    }

    #[test]
    fn default_num_ctx_is_reasonable() {
        let ctx = compute_default_num_ctx();
        assert!((CTX_LOW..=CTX_HIGH).contains(&ctx));
    }

    #[test]
    fn vram_context_tiers_use_twenty_four_k_for_mid_range_hardware() {
        assert_eq!(num_ctx_for_vram(Some(11_999)), 8_192);
        assert_eq!(num_ctx_for_vram(Some(12_000)), 24_576);
        assert_eq!(num_ctx_for_vram(Some(23_999)), 24_576);
        assert_eq!(num_ctx_for_vram(Some(24_000)), 32_768);
        assert_eq!(num_ctx_for_vram(None), 8_192);
    }
}
