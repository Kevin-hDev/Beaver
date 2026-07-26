use super::{apply_frontend_policy, apply_policy};
use crate::services::forecast::selection_policy::{ForecastSelectionMode, ForecastSelectionPolicy};
use crate::services::forecast::types::ForecastRequest;

const MANUAL_MODEL: &str = "chronos-bolt-small";
const OTHER_MODEL: &str = "chronos-bolt-base";

fn request(model: Option<&str>) -> ForecastRequest {
    ForecastRequest {
        data: Some("[]".into()),
        file_path: None,
        data_profile_id: None,
        target_column: "value".into(),
        date_column: "date".into(),
        series_column: None,
        covariate_columns: Vec::new(),
        horizon: 1,
        frequency: "D".into(),
        model: model.map(str::to_string),
        confidence_level: 0.9,
        selection_id: None,
        selection_source: None,
        selection_reason_codes: Vec::new(),
    }
}

fn auto_request(model: Option<&str>) -> ForecastRequest {
    let mut request = request(model);
    request.selection_id = Some(uuid::Uuid::new_v4().to_string());
    request.selection_source =
        Some(crate::services::forecast::provenance_types::ForecastSelectionSource::Auto);
    request
}

fn policy(mode: ForecastSelectionMode) -> ForecastSelectionPolicy {
    ForecastSelectionPolicy {
        mode,
        manual_model_id: Some(MANUAL_MODEL.into()),
        allow_cloud_in_auto: false,
    }
}

#[test]
fn manual_rejects_a_different_requested_model() {
    let mut input = request(Some(OTHER_MODEL));

    assert!(apply_policy(&mut input, policy(ForecastSelectionMode::Manual)).is_err());
    assert_eq!(input.model.as_deref(), Some(OTHER_MODEL));
}

#[test]
fn auto_requires_and_keeps_the_candidate_selected_by_the_llm() {
    let mut missing = auto_request(None);
    assert!(apply_policy(&mut missing, policy(ForecastSelectionMode::Auto)).is_err());

    let mut selected = auto_request(Some(OTHER_MODEL));
    let effective = apply_policy(&mut selected, policy(ForecastSelectionMode::Auto)).unwrap();
    assert_eq!(effective, OTHER_MODEL);
    assert_eq!(selected.model.as_deref(), Some(OTHER_MODEL));
}

#[test]
fn auto_rejects_cloud_when_the_user_did_not_allow_it() {
    let mut selected = auto_request(Some("timegpt-2-mini"));

    assert!(apply_policy(&mut selected, policy(ForecastSelectionMode::Auto)).is_err());
}

#[test]
fn explicit_override_requires_a_structured_user_reason() {
    let mut rejected = auto_request(Some(OTHER_MODEL));
    rejected.selection_source = Some(
        crate::services::forecast::provenance_types::ForecastSelectionSource::ExplicitUserOverride,
    );
    assert!(apply_policy(&mut rejected, policy(ForecastSelectionMode::Auto)).is_err());

    let mut accepted = rejected;
    accepted.selection_reason_codes = vec!["user_requested".into()];
    assert!(apply_policy(&mut accepted, policy(ForecastSelectionMode::Auto)).is_ok());
}

#[test]
fn frontend_auto_choice_is_validated_without_an_llm_ticket() {
    let mut local = request(Some(OTHER_MODEL));
    assert!(apply_frontend_policy(&mut local, policy(ForecastSelectionMode::Auto)).is_ok());

    let mut cloud = request(Some("timegpt-2-mini"));
    assert!(apply_frontend_policy(&mut cloud, policy(ForecastSelectionMode::Auto)).is_err());
}
