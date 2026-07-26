use super::{audit_request_data, DataProfile};
use crate::services::forecast::types::ForecastRequest;
use serde_json::json;

fn request(data: serde_json::Value, horizon: u32, frequency: &str) -> ForecastRequest {
    ForecastRequest {
        data: Some(data.to_string()),
        file_path: None,
        data_profile_id: None,
        target_column: "value".into(),
        date_column: "date".into(),
        series_column: None,
        covariate_columns: Vec::new(),
        horizon,
        frequency: frequency.into(),
        model: None,
        confidence_level: 0.9,
        selection_id: None,
        selection_source: None,
        selection_reason_codes: Vec::new(),
    }
}

fn issue_codes(profile: &DataProfile) -> Vec<&str> {
    profile
        .issues
        .iter()
        .map(|issue| issue.code.as_str())
        .collect()
}

#[test]
fn reports_duplicates_and_unordered_dates_as_errors() {
    let input = request(
        json!([
            {"date": "2026-01-02", "value": 1},
            {"date": "2026-01-01", "value": 2},
            {"date": "2026-01-01", "value": 3}
        ]),
        1,
        "D",
    );
    let (_, profile) = audit_request_data(&input).unwrap();

    assert!(!profile.valid);
    assert!(issue_codes(&profile).contains(&"duplicate_date"));
    assert!(issue_codes(&profile).contains(&"unordered_dates"));
}

#[test]
fn missing_periods_are_visible_but_non_blocking() {
    let input = request(
        json!([
            {"date": "2026-01-01", "value": 1},
            {"date": "2026-01-03", "value": 2}
        ]),
        1,
        "D",
    );
    let (_, profile) = audit_request_data(&input).unwrap();

    assert!(profile.valid);
    assert_eq!(profile.missing_periods, 1);
    assert!(issue_codes(&profile).contains(&"missing_periods"));
}

#[test]
fn business_frequency_does_not_report_weekend_gap() {
    let input = request(
        json!([
            {"date": "2026-07-17", "value": 1},
            {"date": "2026-07-20", "value": 2}
        ]),
        1,
        "B",
    );
    let (_, profile) = audit_request_data(&input).unwrap();

    assert!(profile.valid);
    assert_eq!(profile.missing_periods, 0);
}

#[test]
fn rejects_ambiguous_decimal_text() {
    let input = request(
        json!([
            {"date": "2026-01-01", "value": "1,234.56"},
            {"date": "2026-01-02", "value": 2}
        ]),
        1,
        "D",
    );
    let (_, profile) = audit_request_data(&input).unwrap();

    assert!(!profile.valid);
    assert!(issue_codes(&profile).contains(&"invalid_numeric_value"));
}

#[test]
fn reports_categorical_covariates_without_rejecting_compatible_models() {
    let mut request = request(
        json!([
            {"date": "2026-01-01", "value": 1, "weather": "sunny"},
            {"date": "2026-01-02", "value": 2, "weather": "rainy"}
        ]),
        1,
        "D",
    );
    request.covariate_columns = vec!["weather".into()];

    let (_, profile) = audit_request_data(&request).unwrap();

    assert!(profile.valid);
    assert!(profile
        .issues
        .iter()
        .any(|issue| issue.code == "categorical_covariate"));
}

#[test]
fn profile_preserves_the_requested_confidence_contract() {
    let mut input = request(
        json!([
            {"date": "2026-01-01", "value": 1},
            {"date": "2026-01-02", "value": 2}
        ]),
        1,
        "D",
    );
    input.confidence_level = 0.92;

    let (_, profile) = audit_request_data(&input).unwrap();

    assert_eq!(profile.confidence_level, Some(0.92));
}
