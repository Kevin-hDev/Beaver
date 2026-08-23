use super::*;
use crate::services::llm::fast_mode::FastModeRequest;

fn request(fast_mode: FastModeRequest) -> CodexRequest {
    crate::services::codex_client::request::build_codex_request(
        "gpt-5.6-sol",
        &[],
        &[],
        None,
        Some("session-test"),
        fast_mode,
    )
}

#[test]
fn websocket_payload_uses_the_current_response_create_envelope() {
    let payload = build_payload(&request(FastModeRequest::Fast)).unwrap();
    let value: serde_json::Value = serde_json::from_str(&payload).unwrap();

    assert_eq!(value["type"], "response.create");
    assert_eq!(value["model"], "gpt-5.6-sol");
    assert_eq!(value["service_tier"], "priority");
    assert_eq!(value["stream"], true);
    assert_eq!(value["tools"], serde_json::json!([]));
    assert_eq!(value["tool_choice"], "auto");
    assert_eq!(value["parallel_tool_calls"], false);
    assert!(value["prompt_cache_key"]
        .as_str()
        .is_some_and(|key| key.starts_with("bv1_")));
}

#[test]
fn websocket_standard_payload_omits_service_tier() {
    let payload = build_payload(&request(FastModeRequest::Standard)).unwrap();
    let value: serde_json::Value = serde_json::from_str(&payload).unwrap();

    assert_eq!(value["type"], "response.create");
    assert!(value.get("service_tier").is_none());
}

#[test]
fn websocket_cooldown_is_bounded_and_expires() {
    let deadline = cooldown_deadline(1_000);

    assert!(cooldown_active(1_001, deadline));
    assert!(!cooldown_active(deadline, deadline));
    assert_eq!(deadline, 1_000 + WEBSOCKET_COOLDOWN_MS);
}

#[test]
fn fallback_knows_whether_partial_output_must_be_cleared() {
    assert!(WebSocketFailure::Unavailable { partial: true }.has_partial_output());
    assert!(!WebSocketFailure::Unavailable { partial: false }.has_partial_output());
    assert!(!WebSocketFailure::Cancelled.has_partial_output());
}
