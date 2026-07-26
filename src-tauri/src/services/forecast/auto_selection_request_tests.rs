use super::auto_selection::select_with_requested_model;
use super::data_quality::DataProfile;
use super::hardware_profile::HardwareProfile;
use std::collections::BTreeMap;

fn model(id: &str, runnable: bool, runtime_ready: bool) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "runnable": runnable,
        "runtime_ready": runtime_ready
    })
}

fn profile() -> DataProfile {
    DataProfile {
        id: "550e8400-e29b-41d4-a716-446655440000".into(),
        created_at: "2026-01-01T00:00:00Z".into(),
        fingerprint: "a".repeat(64),
        valid: true,
        target_column: "value".into(),
        date_column: "date".into(),
        series_column: None,
        covariate_columns: Vec::new(),
        frequency: "M".into(),
        horizon: 12,
        confidence_level: Some(0.92),
        row_count: 100,
        history_points: 100,
        future_rows: 0,
        series_count: 1,
        series_ids: Vec::new(),
        history_points_by_series: BTreeMap::new(),
        start: "2025-01-01".into(),
        end: "2025-12-31".into(),
        missing_periods: 0,
        outlier_count: 0,
        issues: Vec::new(),
    }
}

#[test]
fn auto_excludes_fixed_grid_models_before_the_llm_sees_them() {
    let models = [
        model("chronos-2", true, true),
        model("timesfm-2.5-200m", true, true),
        model("toto-2.0-1b", true, true),
    ];

    let selection = select_with_requested_model(&models, &profile(), false, hardware(), &[], None);

    assert_eq!(selection.candidates.len(), 1);
    assert_eq!(selection.candidates[0].model_id, "chronos-2");
    assert!(selection.candidates[0]
        .reasons
        .contains(&"confidence_supported"));
}

#[test]
fn requested_incompatible_confidence_has_a_precise_exclusion_reason() {
    let models = [model("timesfm-2.5-200m", true, true)];

    let selection = select_with_requested_model(
        &models,
        &profile(),
        false,
        hardware(),
        &[],
        Some("timesfm-2.5-200m"),
    );

    assert!(selection.candidates.is_empty());
    assert_eq!(
        selection.requested_model.unwrap().exclusion_reason,
        Some("confidence_unsupported")
    );
}

fn hardware() -> HardwareProfile {
    HardwareProfile {
        gpu_memory_kind: super::hardware_profile::GpuMemoryKind::Dedicated,
        vram_total_mb: Some(64_000),
        vram_available_mb: Some(64_000),
        ram_available_mb: Some(64_000),
    }
}

#[test]
fn explicitly_requested_runtime_must_already_be_ready() {
    let models = [
        model("chronos-bolt-tiny", true, true),
        model("moirai-2.0-r-small", true, false),
    ];
    let selection = select_with_requested_model(
        &models,
        &profile(),
        false,
        hardware(),
        &[],
        Some("moirai-2.0-r-small"),
    );

    assert_eq!(selection.candidates.len(), 1);
    assert_eq!(selection.candidates[0].model_id, "chronos-bolt-tiny");
    let requested = selection.requested_model.unwrap();
    assert_eq!(requested.status, "excluded");
    assert_eq!(requested.exclusion_reason, Some("runtime_not_ready"));
    assert!(!requested.runtime_setup_required);
}

#[test]
fn explicitly_requested_unprepared_model_has_a_clear_reason() {
    let models = [model("moirai-2.0-r-small", false, false)];
    let selection = select_with_requested_model(
        &models,
        &profile(),
        false,
        hardware(),
        &[],
        Some("moirai-2.0-r-small"),
    );

    assert!(selection.candidates.is_empty());
    let requested = selection.requested_model.unwrap();
    assert_eq!(requested.status, "excluded");
    assert_eq!(requested.exclusion_reason, Some("runtime_not_ready"));
}

#[test]
fn explicitly_requested_model_is_kept_in_the_shortlist() {
    let models = [
        model("chronos-bolt-tiny", true, true),
        model("chronos-bolt-mini", true, true),
        model("chronos-bolt-small", true, true),
        model("chronos-bolt-base", true, true),
        model("kairos-10m", true, true),
        model("chronos-2", true, true),
    ];
    let selection = select_with_requested_model(
        &models,
        &profile(),
        false,
        hardware(),
        &[],
        Some("chronos-2"),
    );

    assert_eq!(selection.candidates.len(), 5);
    assert_eq!(selection.candidates[0].model_id, "chronos-2");
    assert_eq!(selection.candidates[0].compatibility, "requested");
}

#[test]
fn explicitly_requested_model_still_obeys_resource_safety() {
    let models = [model("chronos-2", true, true)];
    let insufficient = HardwareProfile {
        gpu_memory_kind: super::hardware_profile::GpuMemoryKind::Dedicated,
        vram_total_mb: Some(1),
        vram_available_mb: Some(1),
        ram_available_mb: Some(1),
    };
    let selection = select_with_requested_model(
        &models,
        &profile(),
        false,
        insufficient,
        &[],
        Some("chronos-2"),
    );

    assert!(selection.candidates.is_empty());
    assert_eq!(
        selection.requested_model.unwrap().exclusion_reason,
        Some("resources_insufficient")
    );
}
