use super::snapshot::CompressionSnapshot;
use crate::services::agent_local::types_message::AgentMessage;
use crate::services::agent_local::types_ollama::{ChatMessage, ToolCallFunction, ToolCallOllama};

pub fn project(snapshot: &CompressionSnapshot, persisted: &[AgentMessage]) -> Vec<ChatMessage> {
    let mut runtime = snapshot.canonical_messages.clone();
    let base = runtime.len();
    runtime.extend(
        persisted
            .iter()
            .map(|message| to_chat_message(snapshot, message)),
    );
    let boundary = persisted.iter().position(|message| {
        message.message_kind
            == Some(
                crate::services::agent_local::types_message::AgentMessageKind::CompressionBoundary,
            )
    });
    if let Some(index) = boundary.map(|index| base + index) {
        let barrier = (index + 1).min(runtime.len().saturating_sub(1));
        if let Some(message) = runtime.get_mut(barrier) {
            message.continuity_barrier_before = true;
        }
    }
    runtime
}

fn to_chat_message(snapshot: &CompressionSnapshot, message: &AgentMessage) -> ChatMessage {
    let tool_calls = message.tool_calls.as_ref().map(|calls| {
        calls
            .iter()
            .map(|call| ToolCallOllama {
                id: Some(call.id.clone()),
                extra_content: call.extra_content.clone(),
                function: ToolCallFunction {
                    name: call.function.name.clone(),
                    arguments: call.function.arguments.clone(),
                },
            })
            .collect()
    });
    ChatMessage {
        continuity_barrier_before: false,
        role: message.role.clone(),
        content: provider_content(message),
        images: checkpoint_images(snapshot, &message.id),
        tool_calls,
        tool_name: message.tool_name.clone(),
        tool_call_id: message.tool_call_id.clone(),
        display_thinking: message.thinking.clone(),
        continuation: message.continuation.clone(),
        tool_loop_reasoning: None,
    }
}

fn provider_content(message: &AgentMessage) -> String {
    if message.message_kind
        != Some(
            crate::services::agent_local::types_message::AgentMessageKind::CompressionCheckpoint,
        )
    {
        return message.content.clone();
    }
    let Ok(mut body) = serde_json::from_str::<serde_json::Value>(&message.content) else {
        return message.content.clone();
    };
    if let Some(object) = body.as_object_mut() {
        object.remove("metadata");
    }
    serde_json::to_string_pretty(&body).unwrap_or_else(|_| message.content.clone())
}

fn checkpoint_images(snapshot: &CompressionSnapshot, message_id: &str) -> Option<Vec<String>> {
    let images: Vec<String> = snapshot
        .checkpoint_images
        .iter()
        .filter(|image| image.source_message_id == message_id)
        .map(|image| image.provider_payload.clone())
        .collect();
    (!images.is_empty()).then_some(images)
}
