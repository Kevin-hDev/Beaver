use super::polling::{build_gpu_status, PsResponse};
use crate::services::gpu_vram::{GpuMemoryKind, GpuMemorySnapshot};

fn snapshot(kind: GpuMemoryKind, total_mb: u64, used_mb: u64) -> GpuMemorySnapshot {
    GpuMemorySnapshot {
        kind,
        total_mb,
        used_mb,
    }
}

#[test]
fn dedicated_memory_is_reported_as_vram() {
    let json = r#"{"models":[{"name":"qwen3:14b","size":9000000000,"size_vram":9000000000}]}"#;
    let ps: PsResponse = serde_json::from_str(json).unwrap();
    let status = build_gpu_status(
        &ps,
        Some(snapshot(GpuMemoryKind::Dedicated, 24_000, 19_000)),
    );
    assert_eq!(status.accelerator, "VRAM");
    assert_eq!(status.vram_used_mb, 19000);
    assert_eq!(status.vram_total_mb, 24000);
}

#[test]
fn cpu_fallback() {
    let json = r#"{"models":[{"name":"gemma4:e4b","size":3000000000,"size_vram":0}]}"#;
    let ps: PsResponse = serde_json::from_str(json).unwrap();
    assert_eq!(build_gpu_status(&ps, None).accelerator, "CPU");
}

#[test]
fn gpu_with_system_usage_without_total() {
    let json = r#"{"models":[{"name":"qwen3:14b","size":9000000000,"size_vram":5368709120}]}"#;
    let ps: PsResponse = serde_json::from_str(json).unwrap();
    let status = build_gpu_status(&ps, Some(snapshot(GpuMemoryKind::Dedicated, 0, 5_120)));
    assert_eq!(status.accelerator, "VRAM");
    assert_eq!(status.vram_used_mb, 5120);
}

#[test]
fn idle_with_vram_shows_vram() {
    let status = build_gpu_status(
        &PsResponse { models: vec![] },
        Some(snapshot(GpuMemoryKind::Dedicated, 16_000, 0)),
    );
    assert_eq!(status.accelerator, "VRAM");
    assert!(status.model_loaded.is_none());
}

#[test]
fn idle_without_a_measurement_is_not_misreported_as_cpu() {
    let status = build_gpu_status(&PsResponse { models: vec![] }, None);
    assert_eq!(status.accelerator, "");
    assert!(status.model_loaded.is_none());
}

#[test]
fn apple_unified_memory_is_identified_as_ram() {
    let status = build_gpu_status(
        &PsResponse { models: vec![] },
        Some(snapshot(GpuMemoryKind::Unified, 24_576, 12_288)),
    );

    assert_eq!(status.accelerator, "RAM");
    assert_eq!(status.vram_total_mb, 24_576);
    assert_eq!(status.vram_used_mb, 12_288);
}

#[test]
fn polling_only_observes_and_never_restarts() {
    let source = include_str!("polling.rs");
    assert!(!source.contains("start_sidecar"));
    assert!(!source.contains("restart"));
    assert!(!source.contains("update_ollama"));
}
