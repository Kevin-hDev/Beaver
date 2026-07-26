use crate::services::forecast::selection_policy::ForecastSelectionMode;

#[test]
fn auto_error_keeps_control_with_the_agent() {
    let payload = super::model_error_payload(
        Some(ForecastSelectionMode::Auto),
        "",
        Some("chronos-2"),
        "test error",
    );

    assert_eq!(payload["model_selection"]["mode"], "auto");
    assert_eq!(payload["model_selection"]["selector_locked"], false);
    assert!(payload["model_selection"]["requested_model_ignored"].is_null());
}

#[test]
fn manual_error_reports_the_locked_selector() {
    let payload = super::model_error_payload(
        Some(ForecastSelectionMode::Manual),
        "chronos-2",
        Some("kairos-10m"),
        "test error",
    );

    assert_eq!(payload["model_selection"]["mode"], "manual");
    assert_eq!(payload["model_selection"]["selector_locked"], true);
    assert_eq!(
        payload["model_selection"]["requested_model_ignored"],
        "kairos-10m"
    );
}
