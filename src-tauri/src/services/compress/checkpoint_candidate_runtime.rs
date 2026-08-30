#![allow(
    dead_code,
    reason = "the shared compression orchestrator consumes this staged projection in Task 11"
)]

use super::snapshot::CompressionSnapshot;
use crate::services::agent_local::types_message::AgentMessage;
use crate::services::agent_local::types_ollama::{ChatMessage, ToolCallFunction, ToolCallOllama};

pub fn project(snapshot: &CompressionSnapshot, persisted: &[AgentMessage]) -> Vec<ChatMessage> {
    let mut runtime = snapshot.canonical_messages.clone();
    let base = runtime.len();
    runtime.extend(persisted.iter().map(to_chat_message));
    let boundary = runtime[base..].iter().position(|message| {
        message.role == "assistant" && message.content == super::engine::BOUNDARY_CONTENT
    });
    if let Some(index) = boundary.map(|index| base + index) {
        let barrier = (index + 1).min(runtime.len().saturating_sub(1));
        if let Some(message) = runtime.get_mut(barrier) {
            message.continuity_barrier_before = true;
        }
    }
    runtime
}

fn to_chat_message(message: &AgentMessage) -> ChatMessage {
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
        content: message.content.clone(),
        images: None,
        tool_calls,
        tool_name: message.tool_name.clone(),
        tool_call_id: message.tool_call_id.clone(),
        display_thinking: message.thinking.clone(),
        continuation: message.continuation.clone(),
        tool_loop_reasoning: None,
    }
}
