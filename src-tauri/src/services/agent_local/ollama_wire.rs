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

fn message_value(message: &ChatMessage) -> Value {
    let mut value = Map::new();
    value.insert("role".into(), json!(message.role));
    value.insert("content".into(), json!(message.content));
    insert_optional(&mut value, "images", message.images.as_ref());
    insert_tool_calls(&mut value, message);
    insert_optional(&mut value, "tool_name", message.tool_name.as_ref());
    insert_optional(&mut value, "thinking", message.reasoning_content.as_ref());
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
    use crate::services::agent_local::types_ollama::{
        ToolCallFunction, ToolCallOllama,
    };

    #[test]
    fn native_payload_uses_ollama_thinking_and_strips_api_fields() {
        let message = ChatMessage {
            role: "assistant".into(),
            content: String::new(),
            reasoning_content: Some("raisonnement".into()),
            tool_call_id: Some("call_1".into()),
            tool_calls: Some(vec![ToolCallOllama {
                id: Some("call_1".into()),
                extra_content: Some(json!({"provider": "api"})),
                function: ToolCallFunction {
                    name: "search".into(),
                    arguments: json!({"query": "test"}),
                },
            }]),
            ..Default::default()
        };

        let value = messages_value(&[message]);
        let serialized = value.to_string();
        assert_eq!(value[0]["thinking"], "raisonnement");
        assert!(!serialized.contains("reasoning_content"));
        assert!(!serialized.contains("tool_call_id"));
        assert!(!serialized.contains("extra_content"));
        assert!(!serialized.contains("call_1"));
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
        };
        let value = chat_request(&request, &[]);

        assert_eq!(value["truncate"], false);
        assert!(value.get("think").is_none());
    }
}
