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

#[test]
fn provider_message_wording_cannot_change_codex_classification() {
    let event = serde_json::json!({
        "response": {
            "error": {
                "code": "invalid_request",
                "message": "service unavailable and overloaded"
            }
        }
    });

    assert_eq!(stream_failure(&event), "provider_request_rejected");
}

#[test]
fn codex_payload_too_large_is_not_mislabeled_as_rate_limit() {
    assert_eq!(
        status_error(StatusCode::PAYLOAD_TOO_LARGE, ""),
        "provider_payload_too_large"
    );
}

#[test]
fn codex_structured_service_tier_http_rejection_is_stable() {
    for body in [
        r#"{"error":{"param":"service_tier","code":"invalid_request_error"}}"#,
        r#"{"error":{"code":"unsupported_service_tier"}}"#,
    ] {
        assert_eq!(
            status_error(StatusCode::BAD_REQUEST, body),
            "service_tier_unavailable"
        );
    }
    assert_eq!(
        status_error(
            StatusCode::BAD_REQUEST,
            r#"{"error":{"message":"service tier unavailable"}}"#
        ),
        "provider_request_rejected"
    );
}

#[tokio::test]
async fn codex_http_consumer_returns_the_service_tier_code_without_exposing_body() {
    let body = r#"{"error":{"param":"service_tier","message":"private account detail"}}"#;
    let response = tauri::http::Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(body)
        .unwrap();

    let error = require_success(reqwest::Response::from(response), "gpt-5.6-sol", 128, 1)
        .await
        .unwrap_err();

    assert_eq!(error, "service_tier_unavailable");
    assert!(!error.contains("private account detail"));
}

#[test]
fn codex_responses_error_checks_param_before_the_defensive_code() {
    let by_param = serde_json::json!({
        "response": {"error": {"param": "service_tier", "code": "invalid_request"}}
    });
    let by_code = serde_json::json!({
        "response": {"error": {"code": "unsupported_service_tier"}}
    });
    let wording_only = serde_json::json!({
        "response": {"error": {"message": "service tier unavailable"}}
    });

    assert_eq!(stream_failure(&by_param), "service_tier_unavailable");
    assert_eq!(stream_failure(&by_code), "service_tier_unavailable");
    assert_eq!(stream_failure(&wording_only), "provider_request_rejected");
}
