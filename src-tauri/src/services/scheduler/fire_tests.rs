use super::super::fire_result::{disables_extension_definition, error_code};

#[test]
fn provider_failures_are_stored_as_safe_codes() {
    assert_eq!(error_code("HTTP 401"), "authentication_failed");
    assert_eq!(error_code("model missing"), "model_unavailable");
    assert_eq!(error_code("provider unavailable"), "provider_unavailable");
    assert_eq!(error_code("/private/path exploded"), "failed");
}

#[test]
fn only_a_missing_extension_revokes_durable_consent() {
    assert!(disables_extension_definition("extension_unavailable"));
    for transient in [
        "model_unavailable",
        "project_unavailable",
        "automation_admission_failed",
        "target_session_missing",
    ] {
        assert!(!disables_extension_definition(transient));
    }
}
