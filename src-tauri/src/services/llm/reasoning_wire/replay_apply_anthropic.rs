use super::{ReplayApplyError, ReplayApproval, ReplayEvidence};
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::reasoning_continuity::contract::{ContinuationTarget, ContractId, RouteId};
use crate::services::reasoning_continuity::envelope::ContinuationState;
use crate::services::reasoning_continuity::registry::ReplayRequirement;
use serde_json::Value;

pub(crate) fn apply(
    messages: &[ChatMessage],
    approval: &ReplayApproval<'_>,
    payload_messages: &mut [Value],
) -> Result<(), ReplayApplyError> {
    if approval.adapter != super::AdapterId::AnthropicBlocks
        || approval.envelope.contract_id != ContractId::AnthropicMessagesV1
    {
        return Err(ReplayApplyError::ContractMismatch);
    }
    let ContinuationState::AnthropicBlocks { blocks } = &approval.envelope.continuation else {
        return Err(ReplayApplyError::ContractMismatch);
    };
    validate_tool_links(messages, approval)?;
    let mut payload_index = 0;
    let mut source_index = 0;
    while source_index < messages.len() {
        let message = &messages[source_index];
        match message.role.as_str() {
            "system" | "developer" => source_index += 1,
            "tool" => {
                while source_index < messages.len() && messages[source_index].role == "tool" {
                    source_index += 1;
                }
                payload_index += 1;
            }
            "user" | "assistant" => {
                let payload = payload_messages
                    .get_mut(payload_index)
                    .ok_or(ReplayApplyError::PayloadMismatch)?;
                if message.continuation.as_ref() == Some(approval.envelope) {
                    if message.role != "assistant" || payload["role"] != "assistant" {
                        return Err(ReplayApplyError::PayloadMismatch);
                    }
                    payload["content"] = Value::Array(blocks.clone());
                }
                source_index += 1;
                payload_index += 1;
            }
            _ => return Err(ReplayApplyError::PayloadMismatch),
        }
    }
    (payload_index == payload_messages.len())
        .then_some(())
        .ok_or(ReplayApplyError::PayloadMismatch)
}

pub(crate) fn apply_all(
    messages: &[ChatMessage],
    target: Option<&ContinuationTarget>,
    payload_messages: &mut [Value],
) -> Result<Vec<ReplayEvidence>, ReplayApplyError> {
    let replay_messages = super::messages_after_barrier(messages);
    let Some(target) = super::target_for_request(replay_messages, target) else {
        return Ok(Vec::new());
    };
    let Some(replay_target) = target.replay() else {
        return Ok(Vec::new());
    };
    if replay_target.route_id != RouteId::Anthropic {
        return Ok(Vec::new());
    }
    let policy = crate::services::reasoning_continuity::registry::replay_policy(replay_target)
        .ok_or(ReplayApplyError::Blocked)?;
    if policy.requirement() == ReplayRequirement::Forbidden {
        return Ok(Vec::new());
    }
    let barrier = messages.len().saturating_sub(replay_messages.len());
    let mut evidence = Vec::new();
    for message in messages
        .iter()
        .skip(barrier)
        .filter(|message| message.role == "assistant")
    {
        let Some(envelope) = message.continuation.as_ref() else {
            if policy.requirement() == ReplayRequirement::Required {
                return Err(ReplayApplyError::Blocked);
            }
            continue;
        };
        let approval = super::approval_for_target(&target, envelope)?;
        apply(messages, &approval, payload_messages)?;
        evidence.push(ReplayEvidence::from_message(message)?);
    }
    Ok(evidence)
}

fn validate_tool_links(
    messages: &[ChatMessage],
    approval: &ReplayApproval<'_>,
) -> Result<(), ReplayApplyError> {
    let message = messages
        .iter()
        .find(|message| message.continuation.as_ref() == Some(approval.envelope))
        .ok_or(ReplayApplyError::PayloadMismatch)?;
    let calls = message.tool_calls.as_deref().unwrap_or_default();
    if calls.len() != approval.envelope.tool_links.len() {
        return Err(ReplayApplyError::PayloadMismatch);
    }
    for (call, link) in calls.iter().zip(&approval.envelope.tool_links) {
        if call.id.as_deref() != Some(link.provider_call_id.as_str())
            || call.function.name != link.tool_name
        {
            return Err(ReplayApplyError::PayloadMismatch);
        }
    }
    Ok(())
}
