use crate::services::forecast::auto_selection::select;
use crate::services::forecast::data_quality::DataProfile;
use crate::services::forecast::hardware_profile::HardwareProfile;
use crate::services::forecast::evaluation::types::{
    BacktestIndexResult, BacktestIndexSummary, BacktestKind, BacktestMetrics,
};
use std::collections::BTreeMap;

fn model(id: &str, runnable: bool) -> serde_json::Value {
    serde_json::json!({"id": id, "runnable": runnable, "runtime_ready": runnable})
}

fn profile(series_count: usize, covariates: bool, future_rows: usize) -> DataProfile {
    DataProfile {
        id: "550e8400-e29b-41d4-a716-446655440000".into(),
        created_at: "2026-01-01T00:00:00Z".into(),
        fingerprint: "fingerprint".into(),
        valid: true,
        target_column: "value".into(),
        date_column: "date".into(),
        series_column: (series_count > 1).then(|| "series".into()),
        covariate_columns: if covariates {
            vec!["price".into()]
        } else {
            Vec::new()
        },
        frequency: "D".into(),
        horizon: 12,
        confidence_level: Some(0.8),
        row_count: 100,
        history_points: 100,
        future_rows,
        series_count,
        series_ids: Vec::new(),
        history_points_by_series: BTreeMap::new(),
        start: "2025-01-01".into(),
        end: "2025-12-31".into(),
        missing_periods: 0,
        outlier_count: 0,
        issues: Vec::new(),
    }
}

fn hardware() -> HardwareProfile {
    HardwareProfile {
        gpu_memory_kind: crate::services::forecast::hardware_profile::GpuMemoryKind::Dedicated,
        vram_total_mb: Some(64_000),
        vram_available_mb: Some(64_000),
        ram_available_mb: Some(64_000),
    }
}

#[test]
fn auto_candidates_are_bounded_to_five() {
    let models = [
        "chronos-bolt-tiny",
        "chronos-bolt-mini",
        "chronos-bolt-small",
        "chronos-bolt-base",
        "chronos-2",
        "timesfm-2.5-200m",
        "toto-2.0-4m",
    ]
    .map(|id| model(id, true));

    assert_eq!(
        select(&models, &profile(1, false, 0), false, hardware(), &[])
            .candidates
            .len(),
        5
    );
}

#[test]
fn auto_excludes_cloud_and_non_runnable_models() {
    let models = [
        model("chronos-bolt-tiny", true),
        model("timegpt-2-mini", true),
        model("chronos-bolt-mini", false),
    ];
    let candidates = select(&models, &profile(1, false, 0), false, hardware(), &[]).candidates;

    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].model_id, "chronos-bolt-tiny");
}

#[test]
fn auto_excludes_a_local_runtime_that_is_not_ready() {
    let models = [serde_json::json!({
        "id": "chronos-bolt-tiny",
        "runnable": true,
        "runtime_ready": false
    })];

    let selection = select(&models, &profile(1, false, 0), false, hardware(), &[]);

    assert!(selection.candidates.is_empty());
}

#[test]
fn auto_filters_against_task_capabilities() {
    let models = [
        model("chronos-bolt-tiny", true),
        model("moirai-2.0-r-small", true),
        model("chronos-2", true),
    ];
    let candidates = select(&models, &profile(2, true, 12), false, hardware(), &[]).candidates;

    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].model_id, "chronos-2");
}

#[test]
fn comparable_backtests_drive_the_auto_ranking() {
    let models = [
        model("chronos-bolt-tiny", true),
        model("chronos-bolt-base", true),
    ];
    let evidence = BacktestIndexSummary {
        created_at: "2026-01-02T00:00:00Z".into(),
        horizon: 12,
        windows: 3,
        results: vec![
            backtest("seasonal_naive", BacktestKind::Baseline, 0.7, None),
            backtest("chronos-bolt-tiny", BacktestKind::Model, 0.5, Some(true)),
            backtest("chronos-bolt-base", BacktestKind::Model, 0.8, Some(false)),
        ],
    };

    let selection = select(
        &models,
        &profile(1, false, 0),
        false,
        hardware(),
        &[evidence],
    );

    assert_eq!(selection.basis, "rolling_backtest");
    assert_eq!(selection.candidates[0].model_id, "chronos-bolt-tiny");
    assert_eq!(
        selection.candidates[0]
            .backtest
            .as_ref()
            .and_then(|result| result.beats_best_baseline),
        Some(true)
    );
}

#[test]
fn unknown_hardware_only_keeps_lightweight_safe_models() {
    let models = [
        model("chronos-bolt-tiny", true),
        model("chronos-2", true),
    ];
    let unknown = HardwareProfile {
        gpu_memory_kind: crate::services::forecast::hardware_profile::GpuMemoryKind::Unknown,
        vram_total_mb: None,
        vram_available_mb: None,
        ram_available_mb: None,
    };

    let selection = select(&models, &profile(1, false, 0), false, unknown, &[]);

    assert!(selection
        .candidates
        .iter()
        .all(|candidate| candidate.model_id != "chronos-2"));
}

fn backtest(
    model_id: &str,
    kind: BacktestKind,
    mase: f64,
    beats_best_baseline: Option<bool>,
) -> BacktestIndexResult {
    BacktestIndexResult {
        model_id: model_id.into(),
        kind,
        metrics: Some(BacktestMetrics {
            mase,
            smape: mase,
            mae: mase,
            rmse: mase,
            bias: 0.0,
            stability: 0.0,
            quantile_loss: Some(mase),
        }),
        calibration: None,
        duration_ms: 10,
        max_memory_mb: None,
        beats_best_baseline,
        failure: None,
    }
}
