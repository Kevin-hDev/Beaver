use super::*;
use crate::services::forecast::evaluation::types::{
    BacktestKind, BacktestMetrics, IntervalCalibration,
};

fn analysis(non_negative: bool) -> ForecastResult {
    serde_json::from_value(serde_json::json!({
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "name": "test",
        "target_column": "sales_total",
        "created_at": "2026-01-01T00:00:00Z",
        "model": "chronos-bolt-tiny",
        "provider": "local",
        "horizon": 1,
        "frequency": "D",
        "input_summary": {"points": 4, "start": "2025-01-01", "end": "2025-01-04"},
        "input_data": {"history": [{"date": "2025-01-04", "value": 10.0}]},
        "predictions": [{"date": "2025-01-05", "value": 10.0}],
        "quantiles": {"q10": [9.0], "q50": [10.0], "q90": [11.0]},
        "provenance": {
            "effective_config": {
                "model_parameters": {"non_negative_output": non_negative}
            }
        }
    }))
    .unwrap()
}

fn calibrated_result() -> ModelBacktestResult {
    ModelBacktestResult {
        model_id: "chronos-bolt-tiny".into(),
        kind: BacktestKind::Model,
        metrics: Some(BacktestMetrics {
            mase: 0.8,
            smape: 1.0,
            mae: 1.0,
            rmse: 1.0,
            bias: 0.0,
            stability: 0.0,
            quantile_loss: Some(0.1),
        }),
        calibration: Some(IntervalCalibration {
            theoretical_coverage: 0.9,
            measured_coverage: 0.7,
            mean_width: 2.0,
            residual_half_width: 12.0,
            sample_count: 6,
        }),
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
fn calibration_widens_saved_future_intervals() {
    let mut analysis = analysis(false);

    apply(&mut analysis, &[calibrated_result()]).unwrap();

    assert_eq!(analysis.quantiles.q10, [-2.0]);
    assert_eq!(analysis.quantiles.q90, [22.0]);
}

#[test]
fn calibration_preserves_an_enabled_non_negative_domain() {
    let mut analysis = analysis(true);

    apply(&mut analysis, &[calibrated_result()]).unwrap();

    assert_eq!(analysis.quantiles.q10, [0.0]);
    assert_eq!(analysis.quantiles.q50, [10.0]);
    assert_eq!(analysis.quantiles.q90, [22.0]);
}
