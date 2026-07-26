use super::sensitive_data::{
    redact_high_confidence_string, redact_high_confidence_text,
    redact_json_high_confidence_preserving_shape, redact_json_preserving_shape, redact_string,
};
use super::types_ollama::ChatMessage;
use super::types_session::SubagentLastActivity;

pub fn sanitize_chat_messages(messages: &mut [ChatMessage]) {
    for message in messages {
        if message.role == "tool" {
            redact_string(&mut message.content);
        } else {
            redact_high_confidence_string(&mut message.content);
        }
        if let Some(reasoning) = message.reasoning_content.as_mut() {
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
    redact_json_high_confidence_preserving_shape(value);
    sanitize_embedded_tool_data(value, 0);
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
                redact_json_preserving_shape(value);
                return;
            }
            for key in ["tool_calls", "tool_activities", "tools"] {
                if let Some(tool_data) = map.get_mut(key) {
                    redact_json_preserving_shape(tool_data);
                }
            }
            for item in map.values_mut() {
                sanitize_embedded_tool_data(item, depth + 1);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
#[path = "session_security_tests.rs"]
mod tests;
