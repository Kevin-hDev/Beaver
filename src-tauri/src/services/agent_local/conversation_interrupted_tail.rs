use super::conversation_history_validation::TailState;
use super::types_message::AgentMessage;
use super::types_session::AgentSession;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CloseInterruptedTailError {
    History,
    Capacity,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum RecoveryProof<'a> {
    RecoveredJournal { request_id: &'a str },
    AdmissionFallback { current_execution_id: &'a str },
    #[cfg(test)]
    LegacyWithoutExecution,
}

pub(crate) fn close_recoverable(
    session: &mut AgentSession,
    proof: RecoveryProof<'_>,
) -> Result<bool, CloseInterruptedTailError> {
    let original_len = session.messages.len();
    let user_only_tail = session.messages.last().is_some_and(|message| message.role == "user");
    match super::conversation_history_validation::tail_state(&session.messages)
        .map_err(|_| CloseInterruptedTailError::History)?
    {
        TailState::Terminal if !user_only_tail => return Ok(false),
        TailState::ToolsPending => {
            if !may_close_pending_tools(session, proof) {
                return Err(CloseInterruptedTailError::History);
            }
            super::conversation_interrupted_tools::append_missing_results(session).map_err(
                |error| match error {
                    super::conversation_interrupted_tools::InterruptedToolError::InvalidHistory => {
                        CloseInterruptedTailError::History
                    }
                    super::conversation_interrupted_tools::InterruptedToolError::Capacity => {
                        CloseInterruptedTailError::Capacity
                    }
                },
            )?;
        }
        TailState::Terminal | TailState::ResultsComplete => {}
    }

    let previous = session
        .messages
        .last()
        .ok_or(CloseInterruptedTailError::History)?;
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
        session.messages.truncate(original_len);
        return Err(CloseInterruptedTailError::History);
    }
    Ok(true)
}

fn may_close_pending_tools(session: &AgentSession, proof: RecoveryProof<'_>) -> bool {
    let pending = session
        .messages
        .iter()
        .rfind(|message| message.role == "assistant" && message.tool_calls.is_some())
        .and_then(|message| message.stream_run_id.as_deref());
    let Some(pending) = pending.filter(|id| uuid::Uuid::parse_str(id).is_ok()) else {
        return false;
    };
    match proof {
        RecoveryProof::RecoveredJournal { request_id } => pending == request_id,
        RecoveryProof::AdmissionFallback {
            current_execution_id,
        } => uuid::Uuid::parse_str(current_execution_id).is_ok() && pending != current_execution_id,
        #[cfg(test)]
        RecoveryProof::LegacyWithoutExecution => false,
    }
}
