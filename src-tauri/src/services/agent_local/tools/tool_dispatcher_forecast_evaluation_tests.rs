use super::*;
use serde_json::{json, Value};

#[test]
fn comparison_marks_incompatible_model_backtests_as_partial() {
    let analysis = serde_json::from_value(json!({
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "name": "test",
        "created_at": "2026-01-01T00:00:00Z",
        "model": "chronos-2",
        "provider": "chronos-2",
        "horizon": 1,
        "frequency": "D",
        "input_summary": { "points": 2, "start": "2025-12-30", "end": "2025-12-31" },
        "predictions": [{ "date": "2026-01-01", "value": 1.0 }],
        "quantiles": { "q10": [0.0], "q50": [1.0], "q90": [2.0] },
        "evaluation": {
            "schema_version": 1,
            "created_at": "2026-01-01T00:00:01Z",
            "horizon": 1,
            "windows": 1,
            "results": [{
                "model_id": "timesfm-2.5-200m",
                "kind": "model",
                "metrics": null,
                "calibration": null,
                "folds": [],
                "duration_ms": 0,
                "rank": null,
                "beats_best_baseline": null,
                "warning": "confidence_unsupported",
                "failure": {
                    "code": "confidence_unsupported",
                    "stage": "preflight",
                    "retryable": false
                }
            }]
        }
    }))
    .unwrap();

    let result = comparison_payload(&analysis);
    let payload: Value = serde_json::from_str(&result.content).unwrap();

    assert!(!result.is_error);
    assert_eq!(payload["status"], "partial");
    assert_eq!(
        payload["model_failures"][0]["failure"]["code"],
        "confidence_unsupported"
    );
}
