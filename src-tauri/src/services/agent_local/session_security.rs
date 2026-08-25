use super::sensitive_data::{
    redact_high_confidence_string, redact_high_confidence_text, redact_json_preserving_shape,
    redact_sensitive_json_field, redact_string,
};
use super::types_ollama::ChatMessage;
use super::types_session::SubagentLastActivity;

pub(crate) const MAX_CONTEXT_SNAPSHOT_TOKENS: u32 = 16 * 1024 * 1024;

pub fn sanitize_chat_messages(messages: &mut [ChatMessage]) {
    for message in messages {
        if message.role == "tool" {
            redact_string(&mut message.content);
        } else {
            redact_high_confidence_string(&mut message.content);
        }
        if let Some(reasoning) = message.display_thinking.as_mut() {
            redact_high_confidence_string(reasoning);
        }
        if let Some(reasoning) = message.legacy_tool_loop_reasoning.as_mut() {
            redact_high_confidence_string(reasoning);
        }
        for call in message.tool_calls.iter_mut().flatten() {
            redact_json_preserving_shape(&mut call.function.arguments);
            if let Some(extra) = call.extra_content.as_mut() {
                redact_json_preserving_shape(extra);
            }
        }
    }
}

pub fn sanitize_session_value(value: &mut serde_json::Value) {
    redact_session_json(value, 0, false);
    bound_context_snapshot(value);
    sanitize_embedded_tool_data(value, 0);
}

fn bound_context_snapshot(value: &mut serde_json::Value) {
    let Some(tokens) = value.get_mut("context_tokens") else {
        return;
    };
    let bounded = tokens
        .as_u64()
        .unwrap_or(0)
        .min(MAX_CONTEXT_SNAPSHOT_TOKENS as u64);
    *tokens = serde_json::Value::from(bounded);
}

pub fn redacted_optional(value: &Option<String>) -> Option<String> {
    value.as_deref().map(redact_high_confidence_text)
}

pub fn redacted_activity(value: &Option<SubagentLastActivity>) -> Option<SubagentLastActivity> {
    value.as_ref().map(|activity| SubagentLastActivity {
        kind: redact_high_confidence_text(&activity.kind),
        label: redact_high_confidence_text(&activity.label),
        detail: redacted_optional(&activity.detail),
        updated_at: activity.updated_at,
    })
}

fn sanitize_embedded_tool_data(value: &mut serde_json::Value, depth: usize) {
    if depth > 32 {
        return;
    }
    match value {
        serde_json::Value::Array(items) => {
            for item in items {
                sanitize_embedded_tool_data(item, depth + 1);
            }
        }
        serde_json::Value::Object(map) => {
            let is_tool_message = matches!(
                map.get("role").and_then(serde_json::Value::as_str),
                Some("tool")
            );
            if is_tool_message {
                redact_session_json(value, depth, true);
                return;
            }
            for key in ["tool_calls", "tool_activities", "tools"] {
                if let Some(tool_data) = map.get_mut(key) {
                    redact_session_json(tool_data, depth + 1, true);
                }
            }
            for (key, item) in map {
                if !is_protected_session_field(key) {
                    sanitize_embedded_tool_data(item, depth + 1);
                }
            }
        }
        _ => {}
    }
}

fn redact_session_json(value: &mut serde_json::Value, depth: usize, broad: bool) {
    if depth > 32 {
        redact_json_preserving_shape(value);
        return;
    }
    match value {
        serde_json::Value::String(content) if broad => redact_string(content),
        serde_json::Value::String(content) => redact_high_confidence_string(content),
        serde_json::Value::Array(items) => {
            for item in items {
                redact_session_json(item, depth + 1, broad);
            }
        }
        serde_json::Value::Object(map) => {
            for (key, item) in map {
                if is_protected_session_field(key) {
                    continue;
                }
                if !redact_sensitive_json_field(key, item) {
                    redact_session_json(item, depth + 1, broad);
                }
            }
        }
        _ => {}
    }
}

fn is_protected_session_field(key: &str) -> bool {
    matches!(key, "continuation" | "extra_content" | "id") || key.ends_with("_id")
}

#[cfg(test)]
#[path = "session_security_tests.rs"]
mod tests;
