use super::*;
use crate::services::forecast::types::ForecastResult;
use serde_json::{json, Value};

fn analysis(count: usize) -> ForecastResult {
    let predictions: Vec<Value> = (0..count)
        .map(|index| json!({ "date": format!("2026-01-{:02}", index + 1), "value": index }))
        .collect();
    let values: Vec<usize> = (0..count).collect();
    serde_json::from_value(json!({
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "name": "test",
        "created_at": "2026-01-01T00:00:00Z",
        "model": "chronos-bolt-small",
        "provider": "chronos-bolt",
        "horizon": count,
        "frequency": "D",
        "input_summary": { "points": 2, "start": "2025-12-30", "end": "2025-12-31" },
        "predictions": predictions,
        "quantiles": { "q10": values, "q50": values, "q90": values }
    }))
    .unwrap()
}

#[test]
fn analysis_payload_caps_and_pages_predictions() {
    let json = analysis_payload(&analysis(250), 25, 500).unwrap();
    let payload: Value = serde_json::from_str(&json).unwrap();

    assert_eq!(payload["predictions"].as_array().unwrap().len(), 200);
    assert_eq!(payload["quantiles"]["q10"].as_array().unwrap().len(), 200);
    assert_eq!(payload["pagination"]["offset"], 25);
    assert_eq!(payload["pagination"]["has_more"], true);
}

#[test]
fn list_payload_keeps_a_bounded_latest_slice() {
    let entries: Vec<_> = (0..150)
        .map(|index| analysis(index + 1).to_meta())
        .collect();
    let json = list_payload(&entries).unwrap();
    let payload: Value = serde_json::from_str(&json).unwrap();

    assert_eq!(payload["analyses"].as_array().unwrap().len(), 100);
    assert_eq!(payload["truncated"], true);
}

#[test]
fn analysis_payload_exposes_compact_advanced_evidence() {
    let mut analysis = analysis(2);
    analysis.advanced_analytics = serde_json::from_value(json!({
        "schema_version": 1,
        "generated_at": "2026-07-23T00:00:00Z",
        "decomposition": [{
            "status": "ready", "method": "classical_additive", "period": 7,
            "seasonal_strength": 0.8,
            "points": [{"date": "2026-01-01", "observed": 10.0, "trend": 9.0, "seasonal": 0.5, "residual": 0.5}]
        }],
        "anomalies": [{
            "id": "series-1:0", "date": "2026-01-01", "observed": 10.0,
            "expected": 9.5, "residual": 0.5, "score": 4.0,
            "severity": "medium", "method": "seasonal_robust_residual"
        }],
        "variable_importance": {
            "status": "not_applicable", "method": "chronological_permutation_on_naive_residual",
            "reliability": "unavailable", "validation_points": 0, "items": []
        },
        "drift": []
    })).ok();

    let payload: Value = serde_json::from_str(&analysis_payload(&analysis, 0, 100).unwrap()).unwrap();

    assert_eq!(payload["advanced_analysis"]["residual_anomalies"]["count"], 1);
    assert_eq!(payload["advanced_analysis"]["decomposition"][0]["period"], 7);
    assert!(payload["advanced_analysis"]["decomposition"][0].get("points").is_none());
}
