use super::error_code;

#[test]
fn provider_failures_are_stored_as_safe_codes() {
    assert_eq!(error_code("HTTP 401"), "authentication_failed");
    assert_eq!(error_code("model missing"), "model_unavailable");
    assert_eq!(error_code("provider unavailable"), "provider_unavailable");
    assert_eq!(error_code("/private/path exploded"), "failed");
}
