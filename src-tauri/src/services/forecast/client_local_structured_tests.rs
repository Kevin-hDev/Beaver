use super::*;
use crate::services::forecast::input_data::ParsedInput;
use crate::services::forecast::types::{ForecastRequest, InputSummary};
use serde_json::json;

fn request() -> ForecastRequest {
    ForecastRequest {
        data: Some("[]".into()),
        file_path: None,
        data_profile_id: None,
        target_column: "value".into(),
        date_column: "date".into(),
        series_column: None,
        covariate_columns: Vec::new(),
        horizon: 1,
        frequency: "M".into(),
        model: Some("chronos-2".into()),
        confidence_level: 0.8,
        selection_id: None,
        selection_source: None,
        selection_reason_codes: Vec::new(),
    }
}

#[test]
fn equivalent_model_timestamps_are_normalized_to_expected_dates() {
    let input = ParsedInput {
        values: vec![1.0],
        future_dates: vec!["2026-02-01".into()],
        summary: InputSummary {
            points: 1,
            start: "2026-01-01".into(),
            end: "2026-01-01".into(),
        },
        snapshot: Default::default(),
        history_rows: vec![json!({"date": "2026-01-01", "value": 1.0})],
        future_rows: vec![json!({"date": "2026-02-01", "value": null})],
        data_profile: serde_json::from_value(json!({
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "created_at": "2026-01-01T00:00:00Z",
            "fingerprint": "a".repeat(64),
            "valid": true,
            "target_column": "value",
            "date_column": "date",
            "series_column": null,
            "covariate_columns": [],
            "frequency": "M",
            "horizon": 1,
            "confidence_level": 0.8,
            "row_count": 2,
            "history_points": 1,
            "future_rows": 1,
            "series_count": 1,
            "series_ids": [],
            "history_points_by_series": {"series-1": 1},
            "start": "2026-01-01",
            "end": "2026-01-01",
            "missing_periods": 0,
            "outlier_count": 0,
            "issues": []
        }))
        .unwrap(),
    };
    let body = json!({
        "predictions": [{
            "date": "2026-02-01 00:00:00",
            "series_id": "series-1",
            "value": 2.0,
            "q10": 1.0,
            "q50": 2.0,
            "q90": 3.0
        }]
    });

    let (predictions, _, _, _) = parse(&body, &request(), &input).unwrap();

    assert_eq!(predictions[0].date, "2026-02-01");
}
