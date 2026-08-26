use crate::services::reasoning_continuity::contract::{ContinuationTarget, ReplayTarget};
use crate::services::reasoning_continuity::envelope::{CompletionState, ReasoningEnvelope};

use super::conversation_history::{
    ConversationHistory, ConversationHistoryError, ProviderMessage, ProviderRole,
};
use super::types_message::AgentMessage;

pub(super) fn from_session(
    session: &super::types_session::AgentSession,
    target: &ReplayTarget,
) -> Result<ConversationHistory, ConversationHistoryError> {
    from_continuation(session, &ContinuationTarget::Replay(target.clone()))
}

pub(super) fn from_continuation(
    session: &super::types_session::AgentSession,
    target: &ContinuationTarget,
) -> Result<ConversationHistory, ConversationHistoryError> {
    validate_target(session, target)?;
    super::conversation_history_validation::validate(&session.messages)?;
    validate_envelopes(&session.messages)?;
    let suffix = target.replay().map_or(session.messages.len(), |_| {
        // La transition est l'autorité unique des barrières route/modèle/scope.
        // Elle recule toujours au début du tour concerné, jamais au milieu.
        super::conversation_transition::for_continuation(session, target).compatible_suffix_start
    });
    let mut messages = session
        .messages
        .iter()
        .enumerate()
        .map(|(index, message)| convert(message, index >= suffix, target.replay()))
        .collect::<Result<Vec<_>, _>>()?;
    if suffix > 0 && suffix < messages.len() {
        messages[suffix].continuity_barrier_before = true;
    }
    Ok(ConversationHistory {
        messages,
        compatible_suffix_start: suffix,
    })
}

fn validate_target(
    session: &super::types_session::AgentSession,
    target: &ContinuationTarget,
) -> Result<(), ConversationHistoryError> {
    target.validate().map_err(|_| ConversationHistoryError)?;
    let route = serde_json::to_value(target.route_id())
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned))
        .ok_or(ConversationHistoryError)?;
    let mode = serde_json::to_value(target.reasoning_mode())
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned))
        .ok_or(ConversationHistoryError)?;
    let session_mode = session.reasoning_mode.as_deref().unwrap_or("off");
    if session.provider != route || session.model != target.model_id() || session_mode != mode {
        return Err(ConversationHistoryError);
    }
    Ok(())
}

fn validate_envelopes(messages: &[AgentMessage]) -> Result<(), ConversationHistoryError> {
    for envelope in messages
        .iter()
        .filter_map(|message| message.continuation.as_ref())
    {
        if envelope.validate().is_err()
            || !crate::services::reasoning_continuity::eligibility::state_matches_contract(
                envelope.contract_id,
                &envelope.continuation,
            )
        {
            return Err(ConversationHistoryError);
        }
    }
    Ok(())
}

fn matches_target(envelope: &ReasoningEnvelope, target: &ReplayTarget) -> bool {
    envelope.completion == CompletionState::Complete
        && envelope.source.matches_target(target)
        && crate::services::reasoning_continuity::registry::route_contract(target.route_id)
            == Some(envelope.contract_id)
}

fn convert(
    message: &AgentMessage,
    in_suffix: bool,
    target: Option<&ReplayTarget>,
) -> Result<ProviderMessage, ConversationHistoryError> {
    let role = match message.role.as_str() {
        "user" => ProviderRole::User,
        "assistant" => ProviderRole::Assistant,
        "tool" => ProviderRole::Tool,
        _ => return Err(ConversationHistoryError),
    };
    let continuation = message
        .continuation
        .as_ref()
        .filter(|envelope| {
            target.is_some_and(|target| in_suffix && matches_target(envelope, target))
        })
        .cloned();
    Ok(ProviderMessage {
        message_id: Some(message.id.clone()),
        turn_id: message.turn_id.clone(),
        role,
        content: message.content.clone(),
        images: Vec::new(),
        files: message.files.clone(),
        tool_calls: message.tool_calls.clone(),
        tool_name: message.tool_name.clone(),
        tool_call_id: message.tool_call_id.clone(),
        display_thinking: message.thinking.clone(),
        continuation,
        legacy_tool_loop_reasoning: None,
        skill_id: None,
        skill_name: None,
        continuity_barrier_before: false,
    })
}
