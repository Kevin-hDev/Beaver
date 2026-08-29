use super::{ReplayApplyError, ReplayApproval};
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::reasoning_continuity::contract::ContractId;
use crate::services::reasoning_continuity::envelope::ContinuationState;
use serde_json::Value;

#[cfg(test)]
pub(crate) fn apply_chat_continuity(
    messages: &[ChatMessage],
    approval: &ReplayApproval<'_>,
    payload_messages: &mut [Value],
) -> Result<(), ReplayApplyError> {
    if messages.len() != payload_messages.len() {
        return Err(ReplayApplyError::PayloadMismatch);
    }
    for (message, payload) in messages.iter().zip(payload_messages) {
        if message.continuation.as_ref() != Some(approval.envelope) {
            continue;
        }
        apply_chat_continuity_at(message, approval, payload)?;
    }
    Ok(())
}

pub(crate) fn apply_chat_continuity_at(
    message: &ChatMessage,
    approval: &ReplayApproval<'_>,
    payload: &mut Value,
) -> Result<(), ReplayApplyError> {
    if message.continuation.as_ref() != Some(approval.envelope) || message.role != "assistant" {
        return Err(ReplayApplyError::PayloadMismatch);
    }
    let payload = payload
        .as_object_mut()
        .ok_or(ReplayApplyError::PayloadMismatch)?;
    super::replay_apply_chat::apply(approval, payload)
}

pub(crate) fn apply_chat_payload_continuity(
    approval: &ReplayApproval<'_>,
    payload: &mut Value,
) -> Result<(), ReplayApplyError> {
    let payload = payload
        .as_object_mut()
        .ok_or(ReplayApplyError::PayloadMismatch)?;
    match (approval.adapter, approval.envelope.contract_id) {
        (super::AdapterId::ChatReasoning, ContractId::ZaiChatV1) => {
            payload.insert(
                "thinking".into(),
                serde_json::json!({"type": "enabled", "clear_thinking": false}),
            );
            Ok(())
        }
        (
            super::AdapterId::ChatReasoning,
            ContractId::DeepSeekChatV1 | ContractId::KimiChatV1 | ContractId::QwenChatV1,
        )
        | (super::AdapterId::CerebrasReasoning, ContractId::CerebrasChatV1)
        | (super::AdapterId::GeminiParts, ContractId::GeminiCompatV1)
        | (super::AdapterId::MistralChunks, ContractId::MistralChunksV1) => Ok(()),
        (super::AdapterId::OpenRouterDetails, ContractId::OpenRouterDetailsV1) => {
            let provider = payload
                .entry("provider")
                .or_insert_with(|| serde_json::json!({}))
                .as_object_mut()
                .ok_or(ReplayApplyError::PayloadMismatch)?;
            provider.insert("allow_fallbacks".into(), false.into());
            Ok(())
        }
        _ => Err(ReplayApplyError::ContractMismatch),
    }
}

pub(crate) fn apply_ollama_continuity(
    messages: &[ChatMessage],
    approval: &ReplayApproval<'_>,
    payload_messages: &mut [Value],
) -> Result<(), ReplayApplyError> {
    if approval.adapter != super::AdapterId::OllamaNative
        || approval.envelope.contract_id != ContractId::OllamaNativeV1
        || messages.len() != payload_messages.len()
    {
        return Err(ReplayApplyError::ContractMismatch);
    }
    let ContinuationState::OllamaNative { thinking } = &approval.envelope.continuation else {
        return Err(ReplayApplyError::ContractMismatch);
    };
    for (message, payload) in messages.iter().zip(payload_messages) {
        if message.continuation.as_ref() == Some(approval.envelope) {
            payload
                .as_object_mut()
                .ok_or(ReplayApplyError::PayloadMismatch)?
                .insert("thinking".into(), thinking.clone().into());
        }
    }
    Ok(())
}

pub(crate) fn apply_responses_continuity(
    messages: &[ChatMessage],
    approval: &ReplayApproval<'_>,
    input: &mut Vec<Value>,
) -> Result<(), ReplayApplyError> {
    if approval.adapter != super::AdapterId::ResponsesLocal
        || !matches!(
            approval.envelope.contract_id,
            ContractId::OpenAiResponsesV1
                | ContractId::XaiResponsesV1
                | ContractId::CodexResponsesV1
        )
    {
        return Err(ReplayApplyError::ContractMismatch);
    }
    let ContinuationState::ResponsesLocal { items } = &approval.envelope.continuation else {
        return Err(ReplayApplyError::ContractMismatch);
    };
    if !messages
        .iter()
        .any(|message| message.continuation.as_ref() == Some(approval.envelope))
    {
        return Err(ReplayApplyError::PayloadMismatch);
    }
    input.extend(items.iter().cloned());
    Ok(())
}
