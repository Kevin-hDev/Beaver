use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::agent_local::types_session::{
    AgentMessage, ToolCallRequest, ToolCallRequestFunction,
};
use crate::services::compress::token_estimate;

const SUMMARY_PREFIX: &str = "This session is being continued from a previous conversation";
const CONTEXT_PREFIX: &str = "Recent file context preserved across compression:";

pub fn recent_messages(
    session_messages: &[AgentMessage],
    runtime_messages: &[ChatMessage],
) -> (Vec<ChatMessage>, Vec<AgentMessage>) {
    let runtime_recent = super::state_select::select_chat_tail(runtime_messages);
    let source = merged_session_source(session_messages, runtime_messages);
    let session_recent = super::state_select::select_agent_tail(&source);
    (runtime_recent, session_recent)
}

pub fn tool_chain_is_closed(messages: &[ChatMessage]) -> bool {
    let mut pending = 0usize;
    for message in messages.iter().filter(|message| message.role != "system") {
        if pending > 0 && message.role != "tool" {
            return false;
        }
        if message.role == "assistant" {
            pending = message.tool_calls.as_ref().map_or(0, Vec::len);
        } else if message.role == "tool" && pending > 0 {
            pending -= 1;
        }
    }
    pending == 0
}

pub fn chat_to_agent_message(message: &ChatMessage) -> AgentMessage {
    let tokens = token_estimate::estimate_tokens(std::slice::from_ref(message)) as u32;
    AgentMessage {
        id: uuid::Uuid::new_v4().to_string(),
        role: message.role.clone(),
        content: message.content.clone(),
        thinking: message.reasoning_content.clone(),
        tool_calls: message.tool_calls.as_ref().map(|calls| {
            calls
                .iter()
                .map(|call| ToolCallRequest {
                    extra_content: call.extra_content.clone(),
                    function: ToolCallRequestFunction {
                        name: call.function.name.clone(),
                        arguments: call.function.arguments.clone(),
                    },
                })
                .collect()
        }),
        tool_name: message.tool_name.clone(),
        tool_activities: None,
        segments: None,
        files: vec![],
        timestamp: chrono::Utc::now(),
        tokens,
        work_duration_ms: None,
        skill_names: None,
        stream_run_id: None,
        stream_part: None,
    }
}

pub fn include_chat_message(message: &ChatMessage) -> bool {
    message.role != "system"
        && !is_compress_command(&message.content)
        && !is_compression_context(&message.content)
}

pub fn include_agent_message(message: &AgentMessage) -> bool {
    message.role != "system"
        && !is_compress_command(&message.content)
        && !is_compression_context(&message.content)
}

fn merged_session_source(
    session_messages: &[AgentMessage],
    runtime_messages: &[ChatMessage],
) -> Vec<AgentMessage> {
    let mut merged: Vec<_> = session_messages
        .iter()
        .filter(|message| include_agent_message(message))
        .cloned()
        .collect();
    for message in runtime_messages
        .iter()
        .filter(|message| include_chat_message(message))
    {
        let converted = chat_to_agent_message(message);
        if !merged
            .iter()
            .any(|existing| same_message(existing, &converted))
        {
            merged.push(converted);
        }
    }
    merged
}

fn is_compress_command(content: &str) -> bool {
    content.trim() == "/compress"
}

fn is_compression_context(content: &str) -> bool {
    let trimmed = content.trim_start();
    trimmed.starts_with(SUMMARY_PREFIX) || trimmed.starts_with(CONTEXT_PREFIX)
}

fn same_message(left: &AgentMessage, right: &AgentMessage) -> bool {
    left.role == right.role && left.content == right.content && left.tool_name == right.tool_name
}
