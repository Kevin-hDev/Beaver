use super::{ReplayApplyError, ReplayApproval};
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::reasoning_continuity::contract::ContractId;
use crate::services::reasoning_continuity::envelope::ContinuationState;
use serde_json::Value;

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
    apply_chat_state(approval, message, payload)
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
        (super::AdapterId::ChatReasoning, ContractId::DeepSeekChatV1 | ContractId::KimiChatV1)
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

fn apply_chat_state(
    approval: &ReplayApproval<'_>,
    _message: &ChatMessage,
    payload: &mut serde_json::Map<String, Value>,
) -> Result<(), ReplayApplyError> {
    match (
        approval.adapter,
        approval.envelope.contract_id,
        &approval.envelope.continuation,
    ) {
        (
            super::AdapterId::ChatReasoning,
            ContractId::DeepSeekChatV1,
            ContinuationState::ChatReasoning { reasoning_content },
        ) => {
            payload.insert("reasoning_content".into(), reasoning_content.clone().into());
            Ok(())
        }
        (
            super::AdapterId::ChatReasoning,
            ContractId::KimiChatV1 | ContractId::ZaiChatV1,
            ContinuationState::ChatReasoning { reasoning_content },
        ) => {
            payload.insert("reasoning_content".into(), reasoning_content.clone().into());
            Ok(())
        }
        (
            super::AdapterId::CerebrasReasoning,
            ContractId::CerebrasChatV1,
            ContinuationState::CerebrasReasoning { reasoning },
        ) => {
            let visible = payload
                .get("content")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let content = if approval
                .target
                .model_id
                .to_lowercase()
                .starts_with("gpt-oss")
            {
                format!("{reasoning}{visible}")
            } else {
                format!("<think>{reasoning}</think>{visible}")
            };
            payload.insert("content".into(), content.into());
            Ok(())
        }
        (
            super::AdapterId::GeminiParts,
            ContractId::GeminiCompatV1,
            ContinuationState::GeminiParts { parts },
        ) => apply_gemini_parts(payload, parts),
        (
            super::AdapterId::MistralChunks,
            ContractId::MistralChunksV1,
            ContinuationState::MistralChunks { chunks },
        ) => {
            payload.insert("content".into(), Value::Array(chunks.clone()));
            Ok(())
        }
        (
            super::AdapterId::OpenRouterDetails,
            ContractId::OpenRouterDetailsV1,
            ContinuationState::OpenRouterDetails { details },
        ) => {
            payload.insert("reasoning_details".into(), Value::Array(details.clone()));
            Ok(())
        }
        _ => Err(ReplayApplyError::ContractMismatch),
    }
}

fn apply_gemini_parts(
    payload: &mut serde_json::Map<String, Value>,
    parts: &[Value],
) -> Result<(), ReplayApplyError> {
    for part in parts {
        if let Some(extra_content) = part.get("extra_content") {
            payload.insert("extra_content".into(), extra_content.clone());
            continue;
        }
        let Some(tool_call) = part.get("tool_call") else {
            return Err(ReplayApplyError::PayloadMismatch);
        };
        let index = tool_call["index"]
            .as_u64()
            .and_then(|value| usize::try_from(value).ok())
            .ok_or(ReplayApplyError::PayloadMismatch)?;
        let extra_content = tool_call
            .get("extra_content")
            .ok_or(ReplayApplyError::PayloadMismatch)?;
        let calls = payload
            .get_mut("tool_calls")
            .and_then(Value::as_array_mut)
            .ok_or(ReplayApplyError::PayloadMismatch)?;
        let call = calls
            .get_mut(index)
            .and_then(Value::as_object_mut)
            .ok_or(ReplayApplyError::PayloadMismatch)?;
        call.insert("extra_content".into(), extra_content.clone());
    }
    Ok(())
}
