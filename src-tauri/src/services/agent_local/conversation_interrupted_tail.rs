use super::conversation_history::ConversationHistoryError;
use super::conversation_history_validation::TailState;
use super::types_message::AgentMessage;
use super::types_session::AgentSession;

pub(crate) fn close_recoverable(
    session: &mut AgentSession,
) -> Result<bool, ConversationHistoryError> {
    match super::conversation_history_validation::tail_state(&session.messages)? {
        TailState::Terminal => return Ok(false),
        TailState::ToolsPending => return Err(ConversationHistoryError),
        TailState::ResultsComplete => {}
    }

    let previous = session.messages.last().ok_or(ConversationHistoryError)?;
    let stream_run_id = previous
        .stream_run_id
        .as_ref()
        .filter(|id| uuid::Uuid::parse_str(id).is_ok())
        .cloned();
    let marker = AgentMessage {
        id: uuid::Uuid::new_v4().to_string(),
        turn_id: previous.turn_id.clone(),
        role: "assistant".into(),
        content: String::new(),
        message_kind: None,
        thinking: None,
        tool_calls: None,
        tool_name: None,
        tool_call_id: None,
        continuation: None,
        replay_source: None,
        tool_activities: None,
        segments: None,
        files: Vec::new(),
        timestamp: chrono::Utc::now(),
        tokens: 0,
        work_duration_ms: None,
        skill_names: None,
        skill_ids: None,
        stream_part: stream_run_id.as_ref().map(|_| "final".into()),
        stream_run_id,
    };
    session.messages.push(marker);
    if super::conversation_history_validation::validate(&session.messages).is_err() {
        session.messages.pop();
        return Err(ConversationHistoryError);
    }
    Ok(true)
}
