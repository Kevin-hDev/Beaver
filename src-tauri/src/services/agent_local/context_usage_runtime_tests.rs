use super::*;

#[test]
fn generated_output_uses_cumulative_text_units() {
    let mut result = StreamResult::default();

    assert_eq!(result.record_generated_text("abc"), 1);
    assert_eq!(result.record_generated_text("de"), 2);
    assert_eq!(result.estimated_output_tokens(), 2);
}

#[test]
fn generated_output_includes_tool_call_payload() {
    let mut result = StreamResult::default();
    result.record_generated_tool_call("bash", &serde_json::json!({ "command": "pwd" }));

    assert!(result.estimated_output_tokens() > 0);
}

#[test]
fn generated_output_saturates_at_u32_max() {
    assert_eq!(bounded_tokens(usize::MAX), u32::MAX);
}

#[test]
fn codex_input_excludes_reasoning_that_is_not_replayed() {
    let message = super::super::types_ollama::ChatMessage {
        role: "assistant".into(),
        content: "answer".into(),
        reasoning_content: Some("hidden reasoning".repeat(100)),
        ..Default::default()
    };
    let full = crate::services::compress::token_estimate::estimate_request_tokens(
        std::slice::from_ref(&message),
        &[],
    );

    assert!(prepared_input_tokens("codex-oauth", full, &[message], &[]) < full);
}
