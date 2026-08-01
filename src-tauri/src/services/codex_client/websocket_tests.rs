use super::*;
use crate::services::codex_client::types::ReasoningConfig;

fn request() -> CodexRequest {
    CodexRequest {
        model: "gpt-test".to_string(),
        instructions: "test".to_string(),
        input: vec![serde_json::json!({"role": "user", "content": "bonjour"})],
        stream: true,
        store: false,
        tools: Vec::new(),
        tool_choice: "auto".to_string(),
        parallel_tool_calls: false,
        reasoning: Some(ReasoningConfig {
            effort: "medium".to_string(),
            summary: "auto".to_string(),
        }),
        include: vec!["reasoning.encrypted_content".to_string()],
    }
}

#[test]
fn websocket_payload_uses_the_current_response_create_envelope() {
    let payload = build_payload(&request()).unwrap();
    let value: serde_json::Value = serde_json::from_str(&payload).unwrap();

    assert_eq!(value["type"], "response.create");
    assert_eq!(value["model"], "gpt-test");
    assert_eq!(value["stream"], true);
    assert_eq!(value["parallel_tool_calls"], false);
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
