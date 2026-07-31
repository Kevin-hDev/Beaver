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
fn preserves_real_prompt_count_when_output_count_is_missing() {
    let mut result = StreamResult {
        prompt_tokens: Some(321),
        ..Default::default()
    };
    result.record_generated_text("estimated output");

    let (input, output, estimated) = resolved_usage(999, &result);

    assert_eq!(input, 321);
    assert_eq!(output, result.estimated_output_tokens());
    assert!(estimated);
}
