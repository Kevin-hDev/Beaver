use super::*;
use crate::services::forecast::input_data::parse_request_input;
use crate::services::forecast::types::{ForecastRequest, ForecastResult, Prediction, Quantiles};

fn request() -> ForecastRequest {
    ForecastRequest {
        data: Some(r#"[{"date":"2026-01-01","value":10},{"date":"2026-01-02","value":12}]"#.into()),
        file_path: None,
        data_profile_id: None,
        target_column: "value".into(),
        date_column: "date".into(),
        series_column: None,
        covariate_columns: Vec::new(),
        horizon: 2,
        frequency: "D".into(),
        model: Some("chronos-bolt-small".into()),
        confidence_level: 0.9,
        selection_id: None,
        selection_source: None,
        selection_reason_codes: Vec::new(),
    }
}

fn result(request: &ForecastRequest) -> ForecastResult {
    let input = parse_request_input(request).unwrap();
    ForecastResult {
        schema_version: crate::services::forecast::types::CURRENT_SCHEMA_VERSION,
        revision: crate::services::forecast::types::default_revision(),
        id: "550e8400-e29b-41d4-a716-446655440000".into(),
        name: "test".into(),
        target_column: "value".into(),
        created_at: "2026-01-01T00:00:00Z".into(),
        session_id: None,
        model: "chronos-bolt-small".into(),
        provider: "chronos-bolt".into(),
        horizon: 2,
        frequency: "D".into(),
        confidence_level: 0.9,
        input_summary: input.summary.clone(),
        input_data: input.snapshot.clone(),
        data_profile: Some(input.data_profile),
        predictions: vec![
            Prediction {
                date: "2026-01-03".into(),
                value: 13.0,
                series_id: None,
            },
            Prediction {
                date: "2026-01-04".into(),
                value: 14.0,
                series_id: None,
            },
        ],
        quantiles: Quantiles {
            q10: vec![11.0, 12.0],
            q50: vec![13.0, 14.0],
            q90: vec![15.0, 16.0],
        },
        covariates_used: Vec::new(),
        metrics: None,
        evaluation: None,
        advanced_analytics: None,
        ensemble: None,
        annotations: Vec::new(),
        scenarios: Vec::new(),
        provenance: Default::default(),
    }
}

#[test]
fn accepts_complete_coherent_result() {
    let request = request();
    let input = parse_request_input(&request).unwrap();
    assert!(validate(&result(&request), &request, &input).is_ok());
}

#[test]
fn rejects_partial_predictions_and_quantiles() {
    let request = request();
    let input = parse_request_input(&request).unwrap();
    let mut output = result(&request);
    output.predictions.pop();
    assert!(validate(&output, &request, &input).is_err());

    let mut output = result(&request);
    output.quantiles.q90.pop();
    assert!(validate(&output, &request, &input).is_err());
}

#[test]
fn rejects_wrong_dates_and_interval_order() {
    let request = request();
    let input = parse_request_input(&request).unwrap();
    let mut output = result(&request);
    output.predictions[0].date = "2026-01-05".into();
    assert!(validate(&output, &request, &input).is_err());

    let mut output = result(&request);
    output.quantiles.q10[0] = 20.0;
    assert!(validate(&output, &request, &input).is_err());
}

#[test]
fn rejects_point_outside_its_prediction_interval() {
    let request = request();
    let input = parse_request_input(&request).unwrap();
    let mut output = result(&request);
    output.predictions[0].value = 1_000.0;

    assert!(validate(&output, &request, &input).is_err());

    output.predictions[0].value = 10.0;
    assert!(validate(&output, &request, &input).is_err());
}
