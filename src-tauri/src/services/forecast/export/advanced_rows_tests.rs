use super::common::{clipboard_text, rows, ExportBundle};
use crate::services::forecast::types::ForecastResult;

fn bundle() -> ExportBundle {
    let analysis: ForecastResult = serde_json::from_value(serde_json::json!({
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "name": "Advanced",
        "created_at": "2026-07-23T00:00:00Z",
        "model": "model-a",
        "provider": "local",
        "horizon": 1,
        "frequency": "D",
        "input_summary": {"points": 30, "start": "2026-01-01", "end": "2026-01-30"},
        "predictions": [{"date": "2026-01-31", "value": 10.0}],
        "quantiles": {"q10": [8.0], "q50": [10.0], "q90": [12.0]},
        "advanced_analytics": {
            "schema_version": 1,
            "generated_at": "2026-07-23T00:00:00Z",
            "decomposition": [],
            "anomalies": [{
                "id": "series-1:4", "date": "2026-01-05", "observed": 40.0,
                "expected": 10.0, "residual": 30.0, "score": 5.0,
                "severity": "medium", "method": "seasonal_robust_residual"
            }],
            "variable_importance": {
                "status": "ready", "method": "chronological_permutation_on_naive_residual",
                "reliability": "moderate", "validation_points": 20, "baseline_mae": 3.0,
                "items": [{
                    "name": "price", "score": 2.0, "normalized_score": 1.0,
                    "direction": "negative", "validation_mae": 1.0, "permuted_mae": 3.0
                }]
            },
            "drift": []
        },
        "ensemble": {
            "created_at": "2026-07-23T00:00:00Z",
            "method": "inverse_mase_weighted",
            "validation_status": "members_backtested_ensemble_not_backtested",
            "members": [{"model_id": "model-a", "weight": 0.6, "backtest_mase": 0.5}],
            "predictions": [{"date": "2026-01-31", "value": 11.0}],
            "quantiles": {"q10": [9.0], "q50": [11.0], "q90": [13.0]}
        }
    }))
    .unwrap();
    ExportBundle {
        analysis,
        notes: vec![],
    }
}

#[test]
fn tabular_export_contains_advanced_sections() {
    let rows = rows(&bundle());
    assert!(rows.iter().any(|row| row.section == "residual_anomaly"));
    assert!(rows.iter().any(|row| row.section == "variable_importance"));
    assert!(rows.iter().any(|row| row.section == "ensemble_member"));
}

#[test]
fn text_report_names_methods_and_validation_status() {
    let text = clipboard_text(&bundle());
    assert!(text.contains("ANALYSE AVANCEE"));
    assert!(text.contains("chronological_permutation_on_naive_residual"));
    assert!(text.contains("members_backtested_ensemble_not_backtested"));
}
