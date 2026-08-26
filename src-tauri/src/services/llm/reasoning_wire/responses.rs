use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::reasoning_continuity::contract::{
    ContinuationTarget, ContinuationUse, ReplayTarget, RouteId,
};
use crate::services::reasoning_continuity::registry::ReplayRequirement;
use crate::services::reasoning_continuity::tool_links::ToolLink;
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ResponseItemError {
    UnsupportedItem,
    InvalidFunctionCall,
}

/// Extrait seulement les items Responses qui sont réinjectables tels quels.
/// Un item final inconnu ferme la continuité plutôt que de risquer un rejeu incomplet.
pub(super) fn completed_item(event: &Value) -> Result<Option<Value>, ResponseItemError> {
    if event.get("type").and_then(Value::as_str) != Some("response.output_item.done") {
        return Ok(None);
    }
    event.get("item").map(validate_item).transpose()
}

/// Repli pour les réponses non streamées : l'ordre du tableau provider est conservé.
pub(super) fn final_items(event: &Value) -> Result<Vec<Value>, ResponseItemError> {
    if event.get("type").and_then(Value::as_str) != Some("response.completed") {
        return Ok(Vec::new());
    }
    event
        .pointer("/response/output")
        .and_then(Value::as_array)
        .map(|items| items.iter().map(validate_item).collect())
        .unwrap_or_else(|| Ok(Vec::new()))
}

pub(super) fn tool_link(item: &Value) -> Result<Option<ToolLink>, ResponseItemError> {
    if item.get("type").and_then(Value::as_str) != Some("function_call") {
        return Ok(None);
    }
    let provider_call_id = item
        .get("call_id")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or(ResponseItemError::InvalidFunctionCall)?;
    let tool_name = item
        .get("name")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or(ResponseItemError::InvalidFunctionCall)?;
    Ok(Some(ToolLink {
        provider_call_id: provider_call_id.to_owned(),
        tool_name: tool_name.to_owned(),
    }))
}

/// Prépare une cible pour l'appel qui part réellement : un résultat d'outil
/// impose son contrat `ToolContinuation`, sans perdre le statut debug.
pub(crate) fn target_for_request(
    messages: &[ChatMessage],
    target: Option<&ContinuationTarget>,
) -> Result<Option<ContinuationTarget>, super::replay::ReplayApplyError> {
    let Some(target) = target else {
        return Ok(None);
    };
    let Some(replay) = target.replay() else {
        return Ok(None);
    };
    if !matches!(
        replay.route_id,
        RouteId::OpenAi | RouteId::XaiOauth | RouteId::CodexOauth
    ) {
        return Ok(None);
    }
    let replay = replay_with_use(replay, messages);
    let target = preserve_target_kind(target, replay);
    let policy = crate::services::reasoning_continuity::registry::replay_policy(
        target
            .replay()
            .ok_or(super::replay::ReplayApplyError::Blocked)?,
    )
    .ok_or(super::replay::ReplayApplyError::Blocked)?;
    for message in messages
        .iter()
        .filter(|message| message.role == "assistant")
    {
        let envelope = message
            .continuation
            .as_ref()
            .ok_or(super::replay::ReplayApplyError::Blocked)?;
        super::replay::approval_for_target(&target, envelope)?;
    }
    if policy.requirement == ReplayRequirement::Required
        && messages.iter().any(|message| message.role == "assistant")
        && !messages
            .iter()
            .any(|message| message.continuation.is_some())
    {
        return Err(super::replay::ReplayApplyError::Blocked);
    }
    Ok(Some(target))
}

/// Sérialise les items opaques à l'emplacement exact du tour assistant.
pub(crate) fn items_for_message(
    message: &ChatMessage,
    target: Option<&ContinuationTarget>,
) -> Result<Option<Vec<Value>>, super::replay::ReplayApplyError> {
    let Some(envelope) = message.continuation.as_ref() else {
        return Ok(None);
    };
    let Some(target) = target else {
        return Ok(None);
    };
    let approval = super::replay::approval_for_target(target, envelope)?;
    let crate::services::reasoning_continuity::envelope::ContinuationState::ResponsesLocal {
        items,
    } = &envelope.continuation
    else {
        return Err(super::replay::ReplayApplyError::ContractMismatch);
    };
    if message.role != "assistant" || items.is_empty() {
        return Err(super::replay::ReplayApplyError::PayloadMismatch);
    }
    super::replay::apply_responses_continuity(&[message.clone()], &approval, &mut Vec::new())?;
    Ok(Some(items.clone()))
}

fn replay_with_use(target: &ReplayTarget, messages: &[ChatMessage]) -> ReplayTarget {
    let mut target = target.clone();
    target.continuation_use = if messages
        .last()
        .is_some_and(|message| message.role == "tool")
    {
        ContinuationUse::ToolContinuation
    } else {
        ContinuationUse::UserContinuation
    };
    target
}

fn preserve_target_kind(original: &ContinuationTarget, replay: ReplayTarget) -> ContinuationTarget {
    match original {
        ContinuationTarget::Replay(_) => ContinuationTarget::Replay(replay),
        #[cfg(debug_assertions)]
        ContinuationTarget::FixtureCandidate(_) => ContinuationTarget::FixtureCandidate(replay),
        ContinuationTarget::Forbidden(_) => unreachable!("non replay target was filtered"),
    }
}

fn validate_item(item: &Value) -> Result<Value, ResponseItemError> {
    match item.get("type").and_then(Value::as_str) {
        Some("reasoning" | "message") => Ok(item.clone()),
        Some("function_call") => {
            tool_link(item)?;
            Ok(item.clone())
        }
        _ => Err(ResponseItemError::UnsupportedItem),
    }
}

#[cfg(test)]
#[path = "responses_tests.rs"]
mod tests;
