use serde::Serialize;

use super::checkpoint_document::CheckpointSection;
use crate::services::agent_local::types_message::AgentMessage;

#[derive(Serialize)]
struct RetainedMessage<'a> {
    source_message_id: &'a str,
    content: &'a str,
}

#[derive(Serialize)]
struct RetainedToolMessage<'a> {
    source_message_id: &'a str,
    role: &'a str,
    content: &'a str,
    tool_name: Option<&'a str>,
    tool_call_id: Option<&'a str>,
    tool_calls: Option<&'a [crate::services::agent_local::types_message::ToolCallRequest]>,
}

pub(super) fn append(
    sections: &mut Vec<CheckpointSection>,
    name: &'static str,
    messages: &[AgentMessage],
) -> Result<(), &'static str> {
    if messages.is_empty() {
        return Ok(());
    }
    let retained = messages
        .iter()
        .map(|message| RetainedMessage {
            source_message_id: &message.id,
            content: &message.content,
        })
        .collect::<Vec<_>>();
    sections.push(CheckpointSection {
        name: name.to_string(),
        content: serde_json::to_string(&retained).map_err(|_| "compression_candidate_invalid")?,
    });
    Ok(())
}

pub(super) fn append_tools(
    sections: &mut Vec<CheckpointSection>,
    messages: &[AgentMessage],
) -> Result<(), &'static str> {
    let retained = messages
        .iter()
        .filter(|message| message.role == "tool" || message.tool_calls.is_some())
        .map(|message| RetainedToolMessage {
            source_message_id: &message.id,
            role: &message.role,
            content: &message.content,
            tool_name: message.tool_name.as_deref(),
            tool_call_id: message.tool_call_id.as_deref(),
            tool_calls: message.tool_calls.as_deref(),
        })
        .collect::<Vec<_>>();
    if retained.is_empty() {
        return Ok(());
    }
    sections.push(CheckpointSection {
        name: "retained_tool_results".to_string(),
        content: serde_json::to_string(&retained).map_err(|_| "compression_candidate_invalid")?,
    });
    Ok(())
}
