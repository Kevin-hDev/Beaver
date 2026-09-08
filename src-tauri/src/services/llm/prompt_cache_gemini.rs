use serde_json::{json, Value};

// Keep a 25% margin over Gemini's 4096 minimum; this is an estimate, not a
// tokenizer guarantee. Live-verified 2026-09-08: OpenRouter leaves a marked
// but undersized 1468-token prompt uncached without rejecting generation.
const MIN_REFERENCE_ESTIMATED_TOKENS: usize = 5_120;

pub(super) fn mark_initial_reference(payload: &mut Value) {
    let Some(messages) = payload.get_mut("messages").and_then(Value::as_array_mut) else {
        return;
    };
    for message in messages {
        let is_user = message["role"] == "user";
        if !is_user && !matches!(message["role"].as_str(), Some("system" | "developer")) {
            return;
        }
        if let Some(text) = message["content"].as_str() {
            if crate::services::token_counting::estimate_text_tokens(text)
                >= MIN_REFERENCE_ESTIMATED_TOKENS
            {
                message["content"] = json!([{
                    "type":"text", "text":text,
                    "cache_control":{"type":"ephemeral"}
                }]);
                return;
            }
        }
        // Never move the boundary into assistant reasoning, tool results or
        // subsequent questions: the initial reference must stay reusable.
        if is_user {
            return;
        }
    }
}
