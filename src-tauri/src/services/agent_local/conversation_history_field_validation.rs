use crate::models::agent_turn_contract::MAX_TURN_CONTENT_BYTES;
use crate::services::reasoning_continuity::limits;

use super::conversation_history::ConversationHistoryError;
use super::types_message::AgentMessage;

pub(super) fn validate(message: &AgentMessage) -> Result<(), ConversationHistoryError> {
    if invalid_content(message)
        || message.thinking.as_deref().is_some_and(invalid_session_text)
        || message.files.len() > crate::models::agent_turn_contract::MAX_TURN_ATTACHMENTS
        || message.files.iter().any(|file| {
            super::conversation_attachment_format::validate_persisted(file).is_err()
        })
        || message.replay_source.as_ref().is_some_and(|source| source.validate().is_err())
    {
        return Err(ConversationHistoryError);
    }
    validate_skills(message)?;
    validate_tool_fields(message)
}

fn validate_skills(message: &AgentMessage) -> Result<(), ConversationHistoryError> {
    super::conversation_skills::validate_persisted_references(
        message.skill_ids.as_deref(),
        message.skill_names.as_deref(),
    )
    .map_err(|_| ConversationHistoryError)
}

fn validate_tool_fields(message: &AgentMessage) -> Result<(), ConversationHistoryError> {
    if message.tool_calls.as_ref().is_some_and(|calls| calls.len() > limits::MAX_TOOL_CALLS)
        || message.tool_name.as_deref().is_some_and(|name| limits::validate_tool_name(name).is_err())
        || message.tool_call_id.as_deref().is_some_and(|id| limits::validate_provider_call_id(id).is_err())
    {
        return Err(ConversationHistoryError);
    }
    for call in message.tool_calls.iter().flatten() {
        if limits::validate_provider_call_id(&call.id).is_err()
            || limits::validate_tool_name(&call.function.name).is_err()
            || limits::validate_json_depth(&call.function.arguments).is_err()
            || call.extra_content.as_ref().is_some_and(|value| limits::validate_json_depth(value).is_err())
        {
            return Err(ConversationHistoryError);
        }
    }
    Ok(())
}

fn invalid_content(message: &AgentMessage) -> bool {
    if message.content.contains('\0') {
        return true;
    }
    if message.role == "user" {
        message.content.len() > MAX_TURN_CONTENT_BYTES
    } else {
        invalid_session_text(&message.content)
    }
}

fn invalid_session_text(value: &str) -> bool {
    value.contains('\0')
        || value.len() as u64 > super::session_limits::MAX_SESSION_FILE_BYTES
}
