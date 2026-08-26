use super::sensitive_data::{
    redact_high_confidence_string, redact_high_confidence_text,
    redact_json_high_confidence_preserving_shape, redact_json_preserving_shape, redact_string,
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
    sanitize_session_root(value);
    bound_context_snapshot(value);
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

fn sanitize_session_root(value: &mut serde_json::Value) {
    let Some(session) = value.as_object_mut() else {
        redact_json_high_confidence_preserving_shape(value);
        return;
    };
    for (key, item) in session {
        match key.as_str() {
            "messages" => sanitize_messages(item),
            key if is_session_link(key) => {}
            _ => redact_json_high_confidence_preserving_shape(item),
        }
    }
}

fn sanitize_messages(value: &mut serde_json::Value) {
    let Some(messages) = value.as_array_mut() else {
        redact_json_high_confidence_preserving_shape(value);
        return;
    };
    for message in messages {
        sanitize_message(message);
    }
}

fn sanitize_message(value: &mut serde_json::Value) {
    let Some(message) = value.as_object_mut() else {
        redact_json_high_confidence_preserving_shape(value);
        return;
    };
    let broad = message
        .get("role")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|role| role == "tool");
    for (key, item) in message {
        match key.as_str() {
            "id" | "turn_id" | "tool_call_id" | "stream_run_id" | "continuation"
            | "replay_source" | "skill_ids" => {}
            "tool_calls" => sanitize_tool_calls(item),
            "tool_activities" => redact_json_preserving_shape(item),
            "segments" => sanitize_segments(item),
            _ if broad => redact_json_preserving_shape(item),
            _ => redact_json_high_confidence_preserving_shape(item),
        }
    }
}

fn sanitize_tool_calls(value: &mut serde_json::Value) {
    let Some(calls) = value.as_array_mut() else {
        redact_json_preserving_shape(value);
        return;
    };
    for call in calls {
        let Some(call) = call.as_object_mut() else {
            redact_json_preserving_shape(call);
            continue;
        };
        for (key, item) in call {
            match key.as_str() {
                "id" | "extra_content" => {}
                _ => redact_json_preserving_shape(item),
            }
        }
    }
}

fn sanitize_segments(value: &mut serde_json::Value) {
    redact_json_high_confidence_preserving_shape(value);
    let Some(segments) = value.as_array_mut() else {
        return;
    };
    for segment in segments {
        if let Some(tools) = segment.get_mut("tools") {
            redact_json_preserving_shape(tools);
        }
    }
}

fn is_session_link(key: &str) -> bool {
    matches!(
        key,
        "id" | "active_todo_run_id"
            | "active_plan_id"
            | "gateway_channel_key"
            | "project_id"
            | "parent_session_id"
            | "subagent_run_id"
            | "clone_parent_session_id"
            | "clone_parent_message_id"
            | "clone_root_session_id"
    )
}

#[cfg(test)]
#[path = "session_security_tests.rs"]
mod tests;
