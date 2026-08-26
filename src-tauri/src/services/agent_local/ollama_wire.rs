use super::types_ollama::{ChatMessage, ChatRequest};
use serde_json::{json, Map, Value};

pub fn chat_request(request: &ChatRequest, messages: &[ChatMessage]) -> Value {
    let mut body = Map::new();
    body.insert("model".into(), json!(request.model));
    body.insert("messages".into(), messages_value(messages));
    body.insert("stream".into(), json!(request.stream));
    body.insert("truncate".into(), json!(false));
    insert_optional(&mut body, "tools", request.tools.as_ref());
    insert_optional(&mut body, "options", request.options.as_ref());
    insert_optional(&mut body, "keep_alive", request.keep_alive.as_ref());
    insert_optional(&mut body, "think", request.think.as_ref());
    Value::Object(body)
}

pub fn messages_value(messages: &[ChatMessage]) -> Value {
    Value::Array(messages.iter().map(message_value).collect())
}

/// Réservé au contrat natif `/api/chat`; `chat_request` conserve encore le
/// champ legacy jusqu'à la bascule atomique de Task 19.
#[allow(
    dead_code,
    reason = "Task 19 connects this only after a live-validated native policy"
)]
pub(crate) fn apply_continuity(
    messages: &[ChatMessage],
    approval: &crate::services::llm::reasoning_wire::replay::ReplayApproval<'_>,
    payload_messages: &mut [Value],
) -> Result<(), crate::services::llm::reasoning_wire::replay::ReplayApplyError> {
    crate::services::llm::reasoning_wire::replay::apply_ollama_continuity(
        messages,
        approval,
        payload_messages,
    )
}

fn message_value(message: &ChatMessage) -> Value {
    let mut value = Map::new();
    value.insert("role".into(), json!(message.role));
    value.insert("content".into(), json!(message.content));
    insert_optional(&mut value, "images", message.images.as_ref());
    insert_tool_calls(&mut value, message);
    insert_optional(&mut value, "tool_name", message.tool_name.as_ref());
    insert_optional(
        &mut value,
        "thinking",
        message.legacy_tool_loop_reasoning.as_ref(),
    );
    Value::Object(value)
}

fn insert_tool_calls(value: &mut Map<String, Value>, message: &ChatMessage) {
    let Some(calls) = message.tool_calls.as_ref() else {
        return;
    };
    let calls = calls
        .iter()
        .map(|call| json!({ "function": call.function }))
        .collect();
    value.insert("tool_calls".into(), Value::Array(calls));
}

fn insert_optional<T: serde::Serialize>(
    value: &mut Map<String, Value>,
    key: &str,
    item: Option<&T>,
) {
    if let Some(item) = item {
        value.insert(key.into(), json!(item));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::agent_local::types_ollama::{ToolCallFunction, ToolCallOllama};

    #[test]
    fn native_payload_uses_ollama_thinking_and_strips_local_tool_ids() {
        let mut message = ChatMessage::assistant(
            String::new(),
            Some("raisonnement".into()),
            None,
            Some("raisonnement".into()),
            Some(vec![ToolCallOllama {
                id: Some("0f7a0a1a-0000-4000-8000-000000000001".into()),
                extra_content: Some(json!({"provider": "api"})),
                function: ToolCallFunction {
                    name: "search".into(),
                    arguments: json!({"query": "test"}),
                },
            }]),
        );
        message.tool_call_id = Some("0f7a0a1a-0000-4000-8000-000000000002".into());

        let value = messages_value(&[message]);
        let serialized = value.to_string();
        assert_eq!(value[0]["thinking"], "raisonnement");
        assert!(!serialized.contains("reasoning_content"));
        assert!(!serialized.contains("tool_call_id"));
        assert!(!serialized.contains("extra_content"));
        assert!(!serialized.contains("0f7a0a1a-0000-4000-8000-000000000001"));
        assert!(!serialized.contains("0f7a0a1a-0000-4000-8000-000000000002"));
    }

    #[test]
    fn chat_payload_disables_ollama_truncation() {
        let request = ChatRequest {
            model: "gemma4:e2b".into(),
            messages: Vec::new(),
            stream: true,
            tools: None,
            options: None,
            keep_alive: None,
            think: None,
            capture_reasoning: false,
        };
        let value = chat_request(&request, &[]);

        assert_eq!(value["truncate"], false);
        assert!(value.get("think").is_none());
    }
}
