use super::*;

#[test]
fn early_close_is_retryable() {
    assert_eq!(closed_before_completed(), "provider_connection_failed");
}

#[test]
fn incomplete_response_is_retryable() {
    assert_eq!(incomplete_response(), "provider_temporarily_unavailable");
}

#[test]
fn failed_response_keeps_safe_provider_classification() {
    let event = serde_json::json!({
        "response": { "error": { "code": "invalid_request", "message": "private" } }
    });

    assert_eq!(failed_response(&event), "provider_request_rejected");
}
