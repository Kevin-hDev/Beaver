use super::*;

#[test]
fn codex_503_is_retryable_without_exposing_provider_details() {
    let error = status_error(StatusCode::SERVICE_UNAVAILABLE, "internal provider details");

    assert_eq!(error, "provider_temporarily_unavailable");
    assert!(!error.contains("internal provider details"));
}

#[test]
fn codex_high_demand_stream_failure_is_retryable() {
    let event = serde_json::json!({
        "response": {
            "error": {
                "code": "server_error",
                "message": "We're currently experiencing high demand."
            }
        }
    });

    assert_eq!(stream_failure(&event), "provider_temporarily_unavailable");
}

#[test]
fn codex_permanent_rejection_stays_generic() {
    let event = serde_json::json!({
        "response": {
            "error": {
                "code": "invalid_request",
                "message": "sensitive internal validation detail"
            }
        }
    });

    assert_eq!(stream_failure(&event), "provider_request_rejected");
}
