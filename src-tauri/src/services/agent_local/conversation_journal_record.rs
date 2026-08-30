use chrono::Utc;

use super::super::types_message::{AgentMessage, ToolCallRequest, ToolCallRequestFunction};
use super::super::types_ollama::ChatMessage;

pub(super) fn from_message(
    message: &ChatMessage,
    id: String,
    turn_id: &str,
    request_id: &str,
) -> Result<AgentMessage, String> {
    let tool_calls = message
        .tool_calls
        .as_ref()
        .map(|calls| {
            calls
                .iter()
                .map(|call| {
                    Ok(ToolCallRequest {
                        id: call.id.clone().ok_or_else(super::validation::error)?,
                        extra_content: call.extra_content.clone(),
                        function: ToolCallRequestFunction {
                            name: call.function.name.clone(),
                            arguments: call.function.arguments.clone(),
                        },
                    })
                })
                .collect::<Result<Vec<_>, String>>()
        })
        .transpose()?;
    let continuation = message.continuation.clone();
    Ok(AgentMessage {
        id,
        turn_id: turn_id.to_string(),
        role: message.role.clone(),
        content: message.content.clone(),
        message_kind: None,
        thinking: message.display_thinking.clone(),
        tool_calls,
        tool_name: message.tool_name.clone(),
        tool_call_id: message.tool_call_id.clone(),
        continuation,
        replay_source: None,
        tool_activities: None,
        segments: None,
        files: Vec::new(),
        timestamp: Utc::now(),
        tokens: 0,
        work_duration_ms: None,
        skill_names: None,
        skill_ids: None,
        stream_run_id: Some(request_id.to_string()),
        stream_part: Some("checkpoint".to_string()),
    })
}
