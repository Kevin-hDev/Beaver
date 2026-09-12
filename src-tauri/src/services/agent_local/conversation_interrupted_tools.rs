use std::collections::HashSet;

use super::types_message::AgentMessage;
use super::types_session::AgentSession;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum InterruptedToolError {
    InvalidHistory,
    Capacity,
}

pub(super) fn append_missing_results(
    session: &mut AgentSession,
) -> Result<usize, InterruptedToolError> {
    let call_index = session
        .messages
        .iter()
        .rposition(|message| message.role == "assistant" && has_calls(message))
        .ok_or(InterruptedToolError::InvalidHistory)?;
    if session.messages[call_index + 1..]
        .iter()
        .any(|message| message.role != "tool")
    {
        return Err(InterruptedToolError::InvalidHistory);
    }

    let call_message = &session.messages[call_index];
    let turn_id = call_message.turn_id.clone();
    let timestamp = call_message.timestamp;
    let completed = session.messages[call_index + 1..]
        .iter()
        .filter_map(|message| message.tool_call_id.as_deref())
        .collect::<HashSet<_>>();
    let missing = call_message
        .tool_calls
        .clone()
        .unwrap_or_default()
        .into_iter()
        .filter(|call| !completed.contains(call.id.as_str()))
        .collect::<Vec<_>>();
    if session.messages.len().saturating_add(missing.len()).saturating_add(1)
        > super::session_limits::MAX_MESSAGES_PER_SESSION
    {
        return Err(InterruptedToolError::Capacity);
    }

    let count = missing.len();
    for call in missing {
        session.messages.push(interrupted_result(
            &turn_id,
            timestamp,
            call.id,
            call.function.name,
        ));
    }
    Ok(count)
}

fn has_calls(message: &AgentMessage) -> bool {
    message
        .tool_calls
        .as_ref()
        .is_some_and(|calls| !calls.is_empty())
}

fn interrupted_result(
    turn_id: &str,
    timestamp: chrono::DateTime<chrono::Utc>,
    tool_call_id: String,
    tool_name: String,
) -> AgentMessage {
    let mut result = super::types_tools::ToolResult::cancelled(
        "Beaver stopped before the tool result was saved. The outcome is unknown; verify external state before retrying.",
    );
    result.error = Some(super::tool_result_contract::ToolErrorInfo::new(
        "tool_interrupted",
        super::tool_result_contract::ToolErrorCategory::Cancelled,
        false,
    ));
    AgentMessage {
        id: uuid::Uuid::new_v4().to_string(),
        turn_id: turn_id.to_string(),
        role: "tool".into(),
        content: super::tool_result_model::render(&tool_name, &result),
        message_kind: None,
        thinking: None,
        tool_calls: None,
        tool_name: Some(tool_name),
        tool_call_id: Some(tool_call_id),
        continuation: None,
        replay_source: None,
        tool_activities: None,
        segments: None,
        files: Vec::new(),
        timestamp,
        tokens: 0,
        work_duration_ms: None,
        skill_names: None,
        skill_ids: None,
        stream_run_id: None,
        stream_part: None,
    }
}
