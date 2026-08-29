use serde_json::{json, Value};

use super::BuildError;
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::llm::route_profile::ImageFormat;
use crate::services::llm::tool_schema::ToolNameMap;

#[derive(Debug, PartialEq)]
pub(super) struct ConvertedMessages {
    pub system: Vec<Value>,
    pub messages: Vec<Value>,
}

pub(super) fn convert(
    source: &[ChatMessage],
    tools: &[Value],
) -> Result<ConvertedMessages, BuildError> {
    let names = ToolNameMap::new(tools);
    let mut system = Vec::new();
    let mut messages = Vec::new();
    let mut pending_results = Vec::new();
    for message in source {
        if message.role != "tool" {
            flush_tool_results(&mut messages, &mut pending_results);
        }
        match message.role.as_str() {
            "system" | "developer" => {
                if !message.content.is_empty() {
                    system.push(json!({"type": "text", "text": message.content}));
                }
            }
            "user" => messages.push(user_message(message)?),
            "assistant" => messages.push(assistant_message(message, &names)?),
            "tool" => pending_results.push(tool_result(message)?),
            _ => return Err(BuildError::InvalidMessage),
        }
    }
    flush_tool_results(&mut messages, &mut pending_results);
    Ok(ConvertedMessages { system, messages })
}

fn user_message(message: &ChatMessage) -> Result<Value, BuildError> {
    let images = message.images.as_deref().unwrap_or_default();
    if images.len() > crate::services::llm::vision::MAX_IMAGES_PER_MESSAGE {
        return Err(BuildError::TooManyImages);
    }
    let mut content = Vec::with_capacity(images.len() + 1);
    for image in images {
        content.push(
            crate::services::llm::vision::image_part(image, ImageFormat::AnthropicBlock)
                .map_err(|_| BuildError::InvalidImage)?,
        );
    }
    if !message.content.is_empty() {
        content.push(json!({"type": "text", "text": message.content}));
    }
    if content.is_empty() {
        return Err(BuildError::InvalidMessage);
    }
    Ok(json!({"role": "user", "content": content}))
}

fn assistant_message(message: &ChatMessage, names: &ToolNameMap) -> Result<Value, BuildError> {
    let mut content = Vec::new();
    if !message.content.is_empty() {
        content.push(json!({"type": "text", "text": message.content}));
    }
    for call in message.tool_calls.as_deref().unwrap_or_default() {
        let id = call
            .id
            .as_deref()
            .filter(|value| !value.is_empty())
            .ok_or(BuildError::InvalidMessage)?;
        content.push(json!({
            "type": "tool_use",
            "id": id,
            "name": names.wire_name(&call.function.name),
            "input": call.function.arguments,
        }));
    }
    if content.is_empty() {
        return Err(BuildError::InvalidMessage);
    }
    Ok(json!({"role": "assistant", "content": content}))
}

fn tool_result(message: &ChatMessage) -> Result<Value, BuildError> {
    let id = message
        .tool_call_id
        .as_deref()
        .filter(|value| !value.is_empty())
        .ok_or(BuildError::InvalidMessage)?;
    Ok(json!({
        "type": "tool_result",
        "tool_use_id": id,
        "content": message.content,
        "is_error": result_is_error(&message.content),
    }))
}

fn result_is_error(content: &str) -> bool {
    content
        .lines()
        .next()
        .and_then(|line| serde_json::from_str::<Value>(line).ok())
        .and_then(|value| {
            value
                .get("status")
                .and_then(Value::as_str)
                .map(str::to_owned)
        })
        .is_some_and(|status| matches!(status.as_str(), "error" | "cancelled"))
}

fn flush_tool_results(messages: &mut Vec<Value>, pending: &mut Vec<Value>) {
    if !pending.is_empty() {
        messages.push(json!({"role": "user", "content": std::mem::take(pending)}));
    }
}
