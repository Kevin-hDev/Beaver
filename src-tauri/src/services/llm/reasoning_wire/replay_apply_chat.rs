use super::{ReplayApplyError, ReplayApproval};
use crate::services::reasoning_continuity::contract::ContractId;
use crate::services::reasoning_continuity::envelope::ContinuationState;
use serde_json::Value;

pub(super) fn apply(
    approval: &ReplayApproval<'_>,
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
            // DeepSeek exige le champ pendant une chaîne d'outils, même vide.
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
            if chunks.is_empty() {
                return Err(ReplayApplyError::Blocked);
            }
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
        let tool_call = part
            .get("tool_call")
            .ok_or(ReplayApplyError::PayloadMismatch)?;
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
