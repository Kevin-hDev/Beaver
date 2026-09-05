use base64::{engine::general_purpose::STANDARD, Engine};
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
    previews: Option<&crate::services::agent_local::tool_artifact_preview::ToolResultPreviewBatch>,
) -> Result<ConvertedMessages, BuildError> {
    let names = ToolNameMap::new(tools);
    let mut system = Vec::new();
    let mut messages = Vec::new();
    let mut pending_results = Vec::new();
    for (index, message) in source.iter().enumerate() {
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
            "tool" => pending_results.push(tool_result(
                message,
                previews.filter(|_| is_last_result_for_call(source, index, message)),
            )?),
            _ => return Err(BuildError::InvalidMessage),
        }
    }
    flush_tool_results(&mut messages, &mut pending_results);
    Ok(ConvertedMessages { system, messages })
}

fn is_last_result_for_call(source: &[ChatMessage], index: usize, message: &ChatMessage) -> bool {
    let Some(id) = message.tool_call_id.as_deref() else {
        return false;
    };
    !source[index.saturating_add(1)..]
        .iter()
        .any(|candidate| candidate.role == "tool" && candidate.tool_call_id.as_deref() == Some(id))
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

fn tool_result(
    message: &ChatMessage,
    previews: Option<&crate::services::agent_local::tool_artifact_preview::ToolResultPreviewBatch>,
) -> Result<Value, BuildError> {
    let id = message
        .tool_call_id
        .as_deref()
        .filter(|value| !value.is_empty())
        .ok_or(BuildError::InvalidMessage)?;
    let images = previews
        .into_iter()
        .flat_map(|batch| batch.previews())
        .filter(|preview| preview.tool_call_id.as_deref() == Some(id))
        .filter(|preview| preview.artifact.mime_type.starts_with("image/"))
        .take(crate::services::llm::vision::MAX_IMAGES_PER_MESSAGE)
        .map(|preview| STANDARD.encode(&preview.artifact.bytes))
        .collect::<Vec<_>>();
    let mut value = crate::services::llm::tool_result_projection::anthropic_tool_result(
        id,
        &message.content,
        &images,
    );
    if let Some(content) = value["content"].as_array_mut() {
        for note in previews
            .into_iter()
            .flat_map(|batch| batch.notes())
            .filter(|note| note.tool_call_id.as_deref() == Some(id))
        {
            content.push(json!({"type":"text","text":note.text}));
        }
        if previews.into_iter().any(|batch| {
            batch
                .omitted_sources()
                .iter()
                .any(|source| source.tool_call_id.as_deref() == Some(id))
        }) {
            content.push(json!({
                "type":"text",
                "text":crate::services::llm::tool_result_projection::ADDITIONAL_PREVIEWS_NOTE,
            }));
        }
    }
    value["is_error"] = result_is_error(&message.content).into();
    Ok(value)
}

fn result_is_error(content: &str) -> bool {
    crate::services::agent_local::tool_result_model_compact::rendered_status_is_error(content)
}

fn flush_tool_results(messages: &mut Vec<Value>, pending: &mut Vec<Value>) {
    if !pending.is_empty() {
        messages.push(json!({"role": "user", "content": std::mem::take(pending)}));
    }
}
