use super::*;
use crate::services::gpu_vram::GpuMemorySnapshot;

#[test]
fn snapshot_kind_is_the_only_gpu_memory_kind_authority() {
    let unknown = profile_from_snapshot(
        Some(GpuMemorySnapshot {
            kind: GpuMemoryKind::Unknown,
            total_mb: 32_768,
            used_mb: Some(8_192),
        }),
        Some(20_000),
    );
    assert_eq!(unknown.gpu_memory_kind, GpuMemoryKind::Unknown);
    assert_eq!(unknown.vram_total_mb, Some(32_768));
    assert_eq!(unknown.vram_available_mb, None);

    let unified = profile_from_snapshot(
        Some(GpuMemorySnapshot {
            kind: GpuMemoryKind::Unified,
            total_mb: 32_768,
            used_mb: Some(8_192),
        }),
        Some(20_000),
    );
    assert_eq!(unified.gpu_memory_kind, GpuMemoryKind::Unified);
    assert_eq!(unified.vram_available_mb, Some(20_000));
}

#[test]
fn unknown_dedicated_usage_never_claims_all_vram_is_available() {
    let profile = profile_from_snapshot(
        Some(GpuMemorySnapshot {
            kind: GpuMemoryKind::Dedicated,
            total_mb: 16_384,
            used_mb: None,
        }),
        Some(20_000),
    );

    assert_eq!(profile.vram_total_mb, Some(16_384));
    assert_eq!(profile.vram_available_mb, None);
}

#[test]
fn safety_margin_blocks_tight_resources() {
    assert_eq!(fit(1_000, 1_199), ResourceFit::Insufficient);
    assert_eq!(fit(1_000, 1_200), ResourceFit::Constrained);
    assert_eq!(fit(1_000, 2_000), ResourceFit::Comfortable);
}

#[test]
fn unknown_cpu_memory_does_not_reject_a_cpu_capable_model() {
    let model = super::super::catalog::find_model("chronos-bolt-tiny").unwrap();
    let profile = HardwareProfile {
        gpu_memory_kind: GpuMemoryKind::Unknown,
        vram_total_mb: None,
        vram_available_mb: None,
        ram_available_mb: None,
    };

    assert_eq!(resource_fit(model, profile), ResourceFit::Unknown);
}

#[test]
fn unknown_resources_reject_large_models_at_execution_time() {
    let model = super::super::catalog::find_model("chronos-2").unwrap();
    let profile = HardwareProfile {
        gpu_memory_kind: GpuMemoryKind::Unknown,
        vram_total_mb: None,
        vram_available_mb: None,
        ram_available_mb: None,
    };

    assert!(validate_model_resources_with_profile(model, profile).is_err());
}

#[test]
fn unknown_resources_keep_lightweight_models_available() {
    let model = super::super::catalog::find_model("chronos-bolt-tiny").unwrap();
    let profile = HardwareProfile {
        gpu_memory_kind: GpuMemoryKind::Unknown,
        vram_total_mb: None,
        vram_available_mb: None,
        ram_available_mb: None,
    };

    assert!(validate_model_resources_with_profile(model, profile).is_ok());
}
