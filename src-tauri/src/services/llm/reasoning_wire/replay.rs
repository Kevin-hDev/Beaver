use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::reasoning_continuity::contract::{ContinuationUse, ContractId, ReplayTarget};
use crate::services::reasoning_continuity::eligibility::{self, ReplayDecision};
use crate::services::reasoning_continuity::envelope::{ContinuationState, ReasoningEnvelope};
use crate::services::reasoning_continuity::registry::{AdapterId, ReplayPolicy};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReplayApplyError {
    Blocked,
    ContractMismatch,
    PayloadMismatch,
}

/// Preuve locale produite après la décision centrale. Elle ne sait pas calculer
/// une route, un modèle ou un scope : elle transporte seulement ce qui a été
/// autorisé par le registre.
#[derive(Debug)]
pub(crate) struct ReplayApproval<'a> {
    envelope: &'a ReasoningEnvelope,
    target: &'a ReplayTarget,
    adapter: AdapterId,
}

pub(crate) fn approved<'a>(
    decision: ReplayDecision,
    policy: ReplayPolicy,
    envelope: &'a ReasoningEnvelope,
    target: &'a ReplayTarget,
) -> Result<ReplayApproval<'a>, ReplayApplyError> {
    if decision != ReplayDecision::Allowed {
        return Err(ReplayApplyError::Blocked);
    }
    let Some((contract_id, adapter)) = policy.live_adapter() else {
        return Err(ReplayApplyError::Blocked);
    };
    if contract_id != envelope.contract_id
        || !eligibility::state_matches_contract(contract_id, &envelope.continuation)
        || !envelope.source.matches_target(target)
    {
        return Err(ReplayApplyError::ContractMismatch);
    }
    Ok(ReplayApproval {
        envelope,
        target,
        adapter,
    })
}

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
        let payload = payload
            .as_object_mut()
            .ok_or(ReplayApplyError::PayloadMismatch)?;
        if message.role != "assistant" {
            return Err(ReplayApplyError::PayloadMismatch);
        }
        apply_chat_state(approval, message, payload)?;
    }
    Ok(())
}

pub(crate) fn apply_chat_payload_continuity(
    approval: &ReplayApproval<'_>,
    payload: &mut Value,
) -> Result<(), ReplayApplyError> {
    let payload = payload
        .as_object_mut()
        .ok_or(ReplayApplyError::PayloadMismatch)?;
    match (approval.adapter, approval.envelope.contract_id) {
        (AdapterId::ChatReasoning, ContractId::ZaiChatV1) => {
            // Opt-in explicite : ce champ n'est atteignable qu'avec une preuve live.
            payload.insert(
                "thinking".into(),
                serde_json::json!({"type": "enabled", "clear_thinking": false}),
            );
            Ok(())
        }
        (AdapterId::CerebrasReasoning, ContractId::CerebrasChatV1) => Ok(()),
        (AdapterId::ChatReasoning, ContractId::DeepSeekChatV1 | ContractId::KimiChatV1)
        | (AdapterId::GeminiParts, ContractId::GeminiCompatV1)
        | (AdapterId::MistralChunks, ContractId::MistralChunksV1)
        | (AdapterId::OpenRouterDetails, ContractId::OpenRouterDetailsV1) => Ok(()),
        _ => Err(ReplayApplyError::ContractMismatch),
    }
}

pub(crate) fn apply_ollama_continuity(
    messages: &[ChatMessage],
    approval: &ReplayApproval<'_>,
    payload_messages: &mut [Value],
) -> Result<(), ReplayApplyError> {
    if approval.adapter != AdapterId::OllamaNative
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
    if approval.adapter != AdapterId::ResponsesLocal
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

fn apply_chat_state(
    approval: &ReplayApproval<'_>,
    message: &ChatMessage,
    payload: &mut serde_json::Map<String, Value>,
) -> Result<(), ReplayApplyError> {
    match (
        approval.adapter,
        approval.envelope.contract_id,
        &approval.envelope.continuation,
    ) {
        (
            AdapterId::ChatReasoning,
            ContractId::DeepSeekChatV1,
            ContinuationState::ChatReasoning { reasoning_content },
        ) => {
            if approval.target.continuation_use != ContinuationUse::ToolContinuation
                || message.tool_calls.is_none()
            {
                return Err(ReplayApplyError::Blocked);
            }
            payload.insert("reasoning_content".into(), reasoning_content.clone().into());
            Ok(())
        }
        (
            AdapterId::ChatReasoning,
            ContractId::KimiChatV1 | ContractId::ZaiChatV1,
            ContinuationState::ChatReasoning { reasoning_content },
        ) => {
            payload.insert("reasoning_content".into(), reasoning_content.clone().into());
            Ok(())
        }
        (
            AdapterId::CerebrasReasoning,
            ContractId::CerebrasChatV1,
            ContinuationState::CerebrasReasoning { reasoning },
        ) => {
            payload.insert("reasoning".into(), reasoning.clone().into());
            Ok(())
        }
        (
            AdapterId::GeminiParts,
            ContractId::GeminiCompatV1,
            ContinuationState::GeminiParts { parts },
        ) => {
            payload.insert("content".into(), Value::Array(parts.clone()));
            Ok(())
        }
        (
            AdapterId::MistralChunks,
            ContractId::MistralChunksV1,
            ContinuationState::MistralChunks { chunks },
        ) => {
            payload.insert("content".into(), Value::Array(chunks.clone()));
            Ok(())
        }
        (
            AdapterId::OpenRouterDetails,
            ContractId::OpenRouterDetailsV1,
            ContinuationState::OpenRouterDetails { details },
        ) => {
            payload.insert("reasoning_details".into(), Value::Array(details.clone()));
            Ok(())
        }
        _ => Err(ReplayApplyError::ContractMismatch),
    }
}

#[cfg(test)]
#[path = "replay_tests.rs"]
mod tests;
