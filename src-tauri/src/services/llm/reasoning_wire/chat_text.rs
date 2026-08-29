use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::reasoning_continuity::contract::{ContinuationTarget, ContractId, RouteId};
use crate::services::reasoning_continuity::registry::ReplayRequirement;
use serde_json::Value;

pub(super) fn fragments(event: &Value, contract: ContractId) -> Vec<&str> {
    let Some(delta) = event.pointer("/choices/0/delta") else {
        return event
            .pointer("/choices/0/message")
            .map_or_else(Vec::new, |message| from_message(message, contract));
    };
    from_message(delta, contract)
}

fn from_message(message: &Value, contract: ContractId) -> Vec<&str> {
    let keys: &[&str] = match contract {
        ContractId::CerebrasChatV1 => &["reasoning"],
        ContractId::DeepSeekChatV1
        | ContractId::KimiChatV1
        | ContractId::ZaiChatV1
        | ContractId::QwenChatV1 => &["reasoning_content"],
        _ => &[],
    };
    keys.iter()
        .copied()
        .filter_map(|key| message.get(key).and_then(Value::as_str))
        .collect()
}

/// Applique une enveloppe native au message assistant exact. Les politiques
/// optionnelles tolèrent seulement l'absence d'enveloppe ; toute provenance
/// présente mais invalide ferme l'appel avant réseau.
pub(crate) fn apply_continuity(
    messages: &[ChatMessage],
    target: Option<&ContinuationTarget>,
    payload: &mut Value,
) -> Result<Vec<super::replay::ReplayEvidence>, super::replay::ReplayApplyError> {
    let replay_messages = super::replay::messages_after_barrier(messages);
    let Some(target) = super::replay::target_for_request(replay_messages, target) else {
        return Ok(Vec::new());
    };
    let Some(replay_target) = target.replay() else {
        return Ok(Vec::new());
    };
    if !is_chat_route(replay_target.route_id) {
        return Ok(Vec::new());
    }
    let Some(policy) =
        crate::services::reasoning_continuity::registry::replay_policy(replay_target)
    else {
        return Err(super::replay::ReplayApplyError::Blocked);
    };
    if policy.requirement() == ReplayRequirement::Forbidden {
        return Ok(Vec::new());
    }
    let mut applied_indexes = Vec::new();
    {
        let payload_messages = payload
            .get_mut("messages")
            .and_then(Value::as_array_mut)
            .ok_or(super::replay::ReplayApplyError::PayloadMismatch)?;
        if payload_messages.len() != messages.len() {
            return Err(super::replay::ReplayApplyError::PayloadMismatch);
        }
        for (index, message) in messages.iter().enumerate() {
            if index + replay_messages.len() < messages.len() {
                continue;
            }
            if message.role != "assistant" {
                continue;
            }
            let Some(envelope) = message.continuation.as_ref() else {
                if policy.requirement() == ReplayRequirement::Required {
                    return Err(super::replay::ReplayApplyError::Blocked);
                }
                continue;
            };
            let approval = super::replay::approval_for_target(&target, envelope)?;
            super::replay::apply_chat_continuity_at(
                message,
                &approval,
                &mut payload_messages[index],
            )?;
            applied_indexes.push(index);
        }
    }
    if let Some(index) = applied_indexes.last().copied() {
        let envelope = messages[index]
            .continuation
            .as_ref()
            .ok_or(super::replay::ReplayApplyError::PayloadMismatch)?;
        let approval = super::replay::approval_for_target(&target, envelope)?;
        super::replay::apply_chat_payload_continuity(&approval, payload)?;
    }
    applied_indexes
        .into_iter()
        .map(|index| super::replay::ReplayEvidence::from_message(&messages[index]))
        .collect()
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
            | RouteId::Qwen
    )
}
