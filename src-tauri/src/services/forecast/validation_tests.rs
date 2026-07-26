use super::types::ForecastRequest;
use super::validation::{validate_model_id, validate_model_id_format, validate_request};

fn make_request(model: &str) -> ForecastRequest {
    ForecastRequest {
        data: Some(r#"[{"date":"2026-05-01","sales":100,"temp":22}]"#.to_string()),
        file_path: None,
        data_profile_id: None,
        target_column: "sales".into(),
        date_column: "date".into(),
        series_column: None,
        covariate_columns: vec!["temp".into()],
        horizon: 3,
        frequency: "D".into(),
        model: Some(model.into()),
        confidence_level: 0.9,
        selection_id: None,
        selection_source: None,
        selection_reason_codes: Vec::new(),
    }
}

#[test]
fn accepts_covariates_for_chronos2() {
    assert!(validate_request(&make_request("chronos-2")).is_ok());
}

#[test]
fn accepts_covariates_for_timesfm_2_5() {
    let mut request = make_request("timesfm-2.5-200m");
    request.confidence_level = 0.8;
    assert!(validate_request(&request).is_ok());
}

#[test]
fn rejects_covariates_for_toto2_until_the_adapter_supports_them() {
    let mut request = make_request("toto-2.0-2.5b");
    request.confidence_level = 0.8;
    assert_eq!(
        validate_request(&request),
        Err("Variables de contexte non supportées par ce moteur".into())
    );
}

#[test]
fn accepts_multiseries_for_timegpt() {
    let mut request = make_request("timegpt-2-mini");
    request.series_column = Some("asset_id".into());

    assert!(validate_request(&request).is_ok());
}

#[test]
fn accepts_independent_multiseries_for_timesfm() {
    let mut request = make_request("timesfm-2.5-200m");
    request.series_column = Some("asset_id".into());
    request.confidence_level = 0.8;

    assert!(validate_request(&request).is_ok());
}

#[test]
fn rejects_multiseries_for_chronos_bolt() {
    let mut request = make_request("chronos-bolt-tiny");
    request.series_column = Some("asset_id".into());
    request.covariate_columns.clear();

    assert_eq!(
        validate_request(&request),
        Err("Multi-séries non supporté par ce moteur".into())
    );
}

#[test]
fn accepts_the_hash_pinned_offline_sundial_model() {
    let mut request = make_request("sundial-128m");
    request.covariate_columns.clear();
    request.confidence_level = 0.8;

    assert!(validate_request(&request).is_ok());
}

#[test]
fn rejects_confidence_unavailable_from_decile_only_model() {
    let mut request = make_request("tirex-35m");
    request.covariate_columns.clear();
    request.confidence_level = 0.9;

    assert_eq!(
        validate_request(&request),
        Err("Niveau de confiance non supporté par ce moteur".into())
    );
    request.confidence_level = 0.8;
    assert!(validate_request(&request).is_ok());
}

#[test]
fn continuous_models_accept_whole_percentage_confidence_levels() {
    let mut request = make_request("chronos-2");
    request.confidence_level = 0.92;
    assert!(validate_request(&request).is_ok());

    request.confidence_level = 0.925;
    assert_eq!(
        validate_request(&request),
        Err("Niveau de confiance invalide".into())
    );
}

#[test]
fn rejects_frequency_outside_the_model_catalog_range() {
    let mut request = make_request("timegpt-2-mini");
    request.frequency = "10S".into();

    assert_eq!(
        validate_request(&request),
        Err("Fréquence non supportée par ce moteur".into())
    );
}

#[test]
fn model_id_format_can_be_checked_before_catalog_resolution() {
    assert!(validate_model_id_format("future-model-1").is_ok());
    assert!(validate_model_id("future-model-1").is_err());
    assert!(validate_model_id_format("../../future-model").is_err());
}
