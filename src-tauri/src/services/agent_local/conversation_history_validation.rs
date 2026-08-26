use std::collections::{HashMap, HashSet};
use std::ops::Range;

use super::conversation_history::ConversationHistoryError;
use super::types_message::AgentMessage;

pub(crate) fn validate(messages: &[AgentMessage]) -> Result<Vec<Range<usize>>, ConversationHistoryError> {
    if messages.len() > super::session_limits::MAX_MESSAGES_PER_SESSION {
        return Err(ConversationHistoryError);
    }
    let mut message_ids = HashSet::with_capacity(messages.len());
    let mut turn_ids = HashSet::with_capacity(messages.len());
    let mut call_ids = HashSet::new();
    let mut turns = Vec::with_capacity(messages.len());
    let mut current = None::<TurnState>;

    for (index, message) in messages.iter().enumerate() {
        validate_common(message, &mut message_ids)?;
        match message.role.as_str() {
            "user" => {
                if let Some(state) = current.take() {
                    if state.phase != Phase::Terminal {
                        return Err(ConversationHistoryError);
                    }
                    turns.push(state.start..index);
                }
                if !turn_ids.insert(message.turn_id.as_str()) || !user_shape(message) {
                    return Err(ConversationHistoryError);
                }
                current = Some(TurnState::new(index, &message.turn_id));
            }
            "assistant" => {
                let state = current.as_mut().ok_or(ConversationHistoryError)?;
                if state.turn_id != message.turn_id
                    || !matches!(state.phase, Phase::User | Phase::ResultsComplete)
                    || !assistant_shape(message)
                {
                    return Err(ConversationHistoryError);
                }
                match message.tool_calls.as_deref() {
                    Some(calls) if !calls.is_empty() => {
                        let mut pending = HashMap::with_capacity(calls.len());
                        for call in calls {
                            if call_ids.len()
                                >= super::session_limits::MAX_MESSAGES_PER_SESSION
                                    * crate::services::reasoning_continuity::limits::MAX_TOOL_CALLS
                                || !call_ids.insert(call.id.as_str())
                                || pending
                                    .insert(call.id.as_str(), call.function.name.as_str())
                                    .is_some()
                            {
                                return Err(ConversationHistoryError);
                            }
                        }
                        validate_tool_links(message, calls)?;
                        state.phase = Phase::ToolsPending(pending);
                    }
                    Some(_) => return Err(ConversationHistoryError),
                    None => {
                        validate_tool_links(message, &[])?;
                        state.phase = Phase::Terminal;
                    }
                }
            }
            "tool" => {
                let state = current.as_mut().ok_or(ConversationHistoryError)?;
                if state.turn_id != message.turn_id || !tool_shape(message) {
                    return Err(ConversationHistoryError);
                }
                let Phase::ToolsPending(pending) = &mut state.phase else {
                    return Err(ConversationHistoryError);
                };
                let call_id = message.tool_call_id.as_deref().ok_or(ConversationHistoryError)?;
                let name = message.tool_name.as_deref().ok_or(ConversationHistoryError)?;
                if pending.remove(call_id) != Some(name) {
                    return Err(ConversationHistoryError);
                }
                if pending.is_empty() {
                    state.phase = Phase::ResultsComplete;
                }
            }
            _ => return Err(ConversationHistoryError),
        }
    }
    if let Some(state) = current {
        if matches!(state.phase, Phase::ToolsPending(_) | Phase::ResultsComplete) {
            return Err(ConversationHistoryError);
        }
        turns.push(state.start..messages.len());
    }
    Ok(turns)
}

fn validate_common(
    message: &AgentMessage,
    ids: &mut HashSet<String>,
) -> Result<(), ConversationHistoryError> {
    super::session_migration_ids::validate_id(&message.id).map_err(|_| ConversationHistoryError)?;
    super::session_migration_ids::validate_id(&message.turn_id)
        .map_err(|_| ConversationHistoryError)?;
    if !ids.insert(message.id.clone())
        || message.validate_stream_metadata().is_err()
    {
        return Err(ConversationHistoryError);
    }
    super::conversation_history_field_validation::validate(message)
}

fn user_shape(message: &AgentMessage) -> bool {
    message.tool_calls.is_none()
        && message.tool_name.is_none()
        && message.tool_call_id.is_none()
        && message.continuation.is_none()
        && message.thinking.is_none()
}

fn assistant_shape(message: &AgentMessage) -> bool {
    message.tool_name.is_none()
        && message.tool_call_id.is_none()
        && message.replay_source.is_none()
}

fn tool_shape(message: &AgentMessage) -> bool {
    message.tool_calls.is_none()
        && message.continuation.is_none()
        && message.thinking.is_none()
        && message.files.is_empty()
        && message.replay_source.is_none()
}

fn validate_tool_links(
    message: &AgentMessage,
    calls: &[super::types_message::ToolCallRequest],
) -> Result<(), ConversationHistoryError> {
    let Some(envelope) = message.continuation.as_ref() else {
        return Ok(());
    };
    if envelope.tool_links.len() != calls.len()
        || calls.iter().any(|call| {
            !envelope.tool_links.iter().any(|link| {
                link.provider_call_id == call.id && link.tool_name == call.function.name
            })
        })
    {
        return Err(ConversationHistoryError);
    }
    Ok(())
}

struct TurnState<'a> {
    start: usize,
    turn_id: &'a str,
    phase: Phase<'a>,
}

impl<'a> TurnState<'a> {
    fn new(start: usize, turn_id: &'a str) -> Self {
        Self { start, turn_id, phase: Phase::User }
    }
}

#[derive(PartialEq, Eq)]
enum Phase<'a> {
    User,
    ToolsPending(HashMap<&'a str, &'a str>),
    ResultsComplete,
    Terminal,
}
