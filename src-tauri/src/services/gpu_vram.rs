#[cfg(target_os = "linux")]
mod linux;
#[cfg(any(test, target_os = "linux"))]
mod linux_drm_snapshot;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(any(test, target_os = "linux", target_os = "windows"))]
mod owned_probe;
#[cfg(test)]
mod owned_probe_tests;
mod snapshot;
#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
mod windows_dxgi;
#[cfg(any(test, target_os = "windows"))]
mod windows_snapshot;

use serde::Serialize;
use snapshot::SnapshotCache;
use std::future::Future;
use std::sync::{Arc, LazyLock};
use std::time::Duration;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GpuMemoryKind {
    Dedicated,
    Unified,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GpuMemorySnapshot {
    pub kind: GpuMemoryKind,
    pub total_mb: u64,
    pub used_mb: Option<u64>,
}

static SNAPSHOT: LazyLock<Arc<SnapshotCache>> =
    LazyLock::new(|| Arc::new(SnapshotCache::default()));
const REFRESH_INTERVAL: Duration = Duration::from_secs(10);

const VRAM_TIER_HIGH_MB: u64 = 24_000;
const VRAM_TIER_MID_MB: u64 = 12_000;
const CTX_HIGH: u32 = 32768;
const CTX_MID: u32 = 24576;
const CTX_LOW: u32 = 8192;

pub fn detect_vram_mb() -> Option<u64> {
    SNAPSHOT.get().map(|snapshot| snapshot.total_mb)
}

pub fn current_snapshot() -> Option<GpuMemorySnapshot> {
    SNAPSHOT.get()
}

#[allow(clippy::needless_return)] // pattern multi-cfg cross-plateforme
pub async fn run_refresh_loop(cancel: crate::services::work_registry::ServiceWorkCancellation) {
    run_refresh_loop_with(
        Arc::clone(&SNAPSHOT),
        cancel,
        REFRESH_INTERVAL,
        measure_owned,
    )
    .await;
}

async fn run_refresh_loop_with<Probe, ProbeFuture>(
    cache: Arc<SnapshotCache>,
    cancel: crate::services::work_registry::ServiceWorkCancellation,
    interval: Duration,
    mut probe: Probe,
) where
    Probe: FnMut(crate::services::work_registry::ServiceWorkCancellation) -> ProbeFuture,
    ProbeFuture: Future<Output = Option<GpuMemorySnapshot>>,
{
    let mut failure_reported = false;
    loop {
        let measurement = probe(cancel.clone()).await;
        if measurement.is_some() {
            if failure_reported {
                ::log::info!("[gpu-memory] probe recovered");
            }
            failure_reported = false;
            cache.publish(measurement);
        } else if !cancel.is_cancelled() && !failure_reported {
            failure_reported = true;
            ::log::warn!("[gpu-memory] probe unavailable; retry scheduled");
        }
        tokio::select! {
            _ = cancel.cancelled() => return,
            _ = tokio::time::sleep(interval) => {}
        }
    }
}

async fn measure_owned(
    cancel: crate::services::work_registry::ServiceWorkCancellation,
) -> Option<GpuMemorySnapshot> {
    detect_owned(&cancel)
        .await
        .map(|(total_mb, used_mb)| GpuMemorySnapshot {
            kind: if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
                GpuMemoryKind::Unified
            } else {
                GpuMemoryKind::Dedicated
            },
            total_mb,
            used_mb,
        })
}

#[allow(clippy::needless_return)] // pattern multi-cfg cross-plateforme
async fn detect_owned(
    cancel: &crate::services::work_registry::ServiceWorkCancellation,
) -> Option<(u64, Option<u64>)> {
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
    use crate::app_exit::AppExitCoordinator;
    use crate::services::work_registry::ServiceWorkSupervisor;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::time::Duration;

    #[test]
    fn published_snapshot_is_the_only_synchronous_measurement_source() {
        SNAPSHOT.replace(Some(GpuMemorySnapshot {
            kind: GpuMemoryKind::Dedicated,
            total_mb: 24_576,
            used_mb: Some(8_192),
        }));

        assert_eq!(detect_vram_mb(), Some(24_576));
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

    #[tokio::test(start_paused = true)]
    async fn refresh_loop_recovers_after_the_initial_probe_fails() {
        let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
        let supervisor = ServiceWorkSupervisor::<1>::new(coordinator.work_supervisor());
        let admission = supervisor.try_admit().expect("refresh admission");
        let cancel = admission.cancellation();
        let cache = Arc::new(SnapshotCache::default());
        let attempts = Arc::new(AtomicUsize::new(0));
        let probe_attempts = Arc::clone(&attempts);
        let loop_cache = Arc::clone(&cache);

        let refresh = tokio::spawn(run_refresh_loop_with(
            loop_cache,
            cancel,
            Duration::from_secs(10),
            move |_| {
                let attempt = probe_attempts.fetch_add(1, Ordering::SeqCst);
                async move {
                    (attempt > 0).then_some(GpuMemorySnapshot {
                        kind: GpuMemoryKind::Dedicated,
                        total_mb: 8_192,
                        used_mb: Some(2_048),
                    })
                }
            },
        ));

        tokio::task::yield_now().await;
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
        assert_eq!(cache.get(), None);

        tokio::time::advance(Duration::from_secs(10)).await;
        tokio::task::yield_now().await;
        assert_eq!(attempts.load(Ordering::SeqCst), 2);
        assert_eq!(cache.get().map(|snapshot| snapshot.total_mb), Some(8_192));

        coordinator.close_work_admission_for_test();
        refresh.await.expect("refresh loop");
        drop(admission);
    }
}
