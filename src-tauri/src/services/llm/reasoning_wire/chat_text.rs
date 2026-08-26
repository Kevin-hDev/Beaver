use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::reasoning_continuity::contract::{ContinuationTarget, RouteId};
use crate::services::reasoning_continuity::registry::ReplayRequirement;
use serde_json::Value;

pub(super) fn fragments(event: &Value) -> Vec<&str> {
    let Some(delta) = event.pointer("/choices/0/delta") else {
        return event
            .pointer("/choices/0/message")
            .map_or_else(Vec::new, from_message);
    };
    from_message(delta)
}

fn from_message(message: &Value) -> Vec<&str> {
    [
        "reasoning_content",
        "reasoning",
        "thought",
        "thought_summary",
    ]
    .into_iter()
    .filter_map(|key| message.get(key).and_then(Value::as_str))
    .collect()
}

/// Applique une enveloppe native au message assistant exact. Les politiques
/// optionnelles restent silencieuses hors activation ; les politiques required
/// ferment l'appel avant réseau dès que l'enveloppe manque ou est invalide.
pub(crate) fn apply_continuity(
    messages: &[ChatMessage],
    target: Option<&ContinuationTarget>,
    payload: &mut Value,
) -> Result<(), super::replay::ReplayApplyError> {
    let Some(target) = super::replay::target_for_request(messages, target) else {
        return Ok(());
    };
    let Some(replay_target) = target.replay() else {
        return Ok(());
    };
    if !is_chat_route(replay_target.route_id) {
        return Ok(());
    }
    let Some(policy) =
        crate::services::reasoning_continuity::registry::replay_policy(replay_target)
    else {
        return Ok(());
    };
    if policy.requirement == ReplayRequirement::Forbidden {
        return Ok(());
    }
    let mut applied_index = None;
    {
        let payload_messages = payload
            .get_mut("messages")
            .and_then(Value::as_array_mut)
            .ok_or(super::replay::ReplayApplyError::PayloadMismatch)?;
        if payload_messages.len() != messages.len() {
            return Err(super::replay::ReplayApplyError::PayloadMismatch);
        }
        for (index, message) in messages.iter().enumerate() {
            if message.role != "assistant" {
                continue;
            }
            let Some(envelope) = message.continuation.as_ref() else {
                if policy.requirement == ReplayRequirement::Required {
                    return Err(super::replay::ReplayApplyError::Blocked);
                }
                continue;
            };
            let approval = match super::replay::approval_for_target(&target, envelope) {
                Ok(approval) => approval,
                Err(_error) if policy.requirement == ReplayRequirement::Optional => continue,
                Err(error) => return Err(error),
            };
            super::replay::apply_chat_continuity_at(
                message,
                &approval,
                &mut payload_messages[index],
            )?;
            applied_index = Some(index);
        }
    }
    if let Some(index) = applied_index {
        let envelope = messages[index]
            .continuation
            .as_ref()
            .ok_or(super::replay::ReplayApplyError::PayloadMismatch)?;
        let approval = super::replay::approval_for_target(&target, envelope)?;
        super::replay::apply_chat_payload_continuity(&approval, payload)?;
    }
    Ok(())
}

fn is_chat_route(route: RouteId) -> bool {
    matches!(
        route,
        RouteId::Google
            | RouteId::Mistral
            | RouteId::Cerebras
            | RouteId::OpenRouter
            | RouteId::DeepSeek
            | RouteId::Moonshot
            | RouteId::MoonshotOauth
            | RouteId::Zai
    )
}
