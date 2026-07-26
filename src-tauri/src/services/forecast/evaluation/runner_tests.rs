use super::*;
use crate::services::forecast::evaluation::types::{BacktestFailure, BacktestMetrics};

fn result(id: &str, kind: BacktestKind, mase: f64) -> ModelBacktestResult {
    ModelBacktestResult {
        model_id: id.into(),
        kind,
        metrics: Some(BacktestMetrics {
            mase,
            smape: 1.0,
            mae: 1.0,
            rmse: 1.0,
            bias: 0.0,
            stability: 0.0,
            quantile_loss: Some(0.1),
        }),
        calibration: None,
        folds: vec![],
        duration_ms: 0,
        max_memory_mb: None,
        rank: None,
        beats_best_baseline: None,
        warning: None,
        failure: None,
    }
}

#[test]
fn advanced_model_must_beat_the_best_baseline() {
    let mut results = vec![
        result("naive", BacktestKind::Baseline, 1.0),
        result("model", BacktestKind::Model, 0.8),
    ];
    rank(&mut results);
    assert_eq!(results[1].rank, Some(1));
    assert_eq!(results[1].beats_best_baseline, Some(true));
}

#[test]
fn invalid_saved_confidence_is_rejected_before_backtesting() {
    let analysis: ForecastResult = serde_json::from_value(serde_json::json!({
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "name": "test",
        "created_at": "2026-01-01T00:00:00Z",
        "model": "chronos-bolt-tiny",
        "provider": "local",
        "horizon": 1,
        "frequency": "D",
        "confidence_level": 0.0,
        "input_summary": {"points": 4, "start": "2025-01-01", "end": "2025-01-04"},
        "predictions": [{"date": "2025-01-05", "value": 10.0}],
        "quantiles": {"q10": [9.0], "q50": [10.0], "q90": [11.0]}
    }))
    .unwrap();

    assert!(validate_analysis(&analysis).is_err());
}

#[test]
fn a_failed_baseline_cannot_validate_the_run() {
    let mut failed = result("naive", BacktestKind::Baseline, 1.0);
    failed.metrics = None;
    failed.failure = Some(BacktestFailure::from_code("invalid_backtest_data"));

    assert!(!has_successful_baseline(&[failed]));
}

#[test]
fn quantile_loss_breaks_equal_mase_ties() {
    let mut weak = result("weak", BacktestKind::Model, 0.8);
    weak.metrics.as_mut().unwrap().quantile_loss = Some(0.4);
    let mut strong = result("strong", BacktestKind::Model, 0.8);
    strong.metrics.as_mut().unwrap().quantile_loss = Some(0.2);
    let mut results = vec![weak, strong];

    rank(&mut results);

    assert_eq!(results[1].rank, Some(1));
}
