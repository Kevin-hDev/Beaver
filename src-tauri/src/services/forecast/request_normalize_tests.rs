use super::normalize_request;
use crate::services::forecast::types::ForecastRequest;
use crate::services::forecast::validation::validate_request;

fn request() -> ForecastRequest {
    ForecastRequest {
        data: Some(r#"[{"date":"2026-05-01","sales":100}]"#.to_string()),
        file_path: Some("  ".into()),
        data_profile_id: None,
        target_column: " sales ".into(),
        date_column: " date ".into(),
        series_column: Some("  ".into()),
        covariate_columns: vec![" ".into(), " temp ".into()],
        horizon: 3,
        frequency: " D ".into(),
        model: Some(" tirex-35m ".into()),
        confidence_level: 0.8,
        selection_id: None,
        selection_source: None,
        selection_reason_codes: Vec::new(),
    }
}

#[test]
fn drops_blank_optional_fields() {
    let mut request = request();

    normalize_request(&mut request);

    assert_eq!(request.file_path, None);
    assert_eq!(request.series_column, None);
    assert_eq!(request.model.as_deref(), Some("tirex-35m"));
}

#[test]
fn trims_columns_and_drops_blank_covariates() {
    let mut request = request();

    normalize_request(&mut request);

    assert_eq!(request.target_column, "sales");
    assert_eq!(request.date_column, "date");
    assert_eq!(request.frequency, "D");
    assert_eq!(request.covariate_columns, vec!["temp"]);
}

#[test]
fn blank_series_column_no_longer_blocks_monoseries() {
    let mut request = request();
    request.covariate_columns.clear();

    normalize_request(&mut request);

    assert!(validate_request(&request).is_ok());
}
