use serde::{Deserialize, Serialize};

use super::contract::{ContractId, ReplayTarget};
use super::envelope::{CompletionState, ContinuationState, ReasoningEnvelope};
use super::registry::{self, ActivationState, ReplayRequirement};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockReason {
    UnknownTarget,
    Forbidden,
    PartialState,
    CompactedState,
    ProvenanceMismatch,
    ContractMismatch,
    InvalidEnvelope,
    NotLiveValidated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplayDecision {
    Allowed,
    Blocked(BlockReason),
}

pub fn decide(envelope: &ReasoningEnvelope, target: &ReplayTarget) -> ReplayDecision {
    if target.validate().is_err() {
        return ReplayDecision::Blocked(BlockReason::UnknownTarget);
    }
    let Some(policy) = registry::replay_policy(target) else {
        return ReplayDecision::Blocked(BlockReason::UnknownTarget);
    };
    if policy.requirement == ReplayRequirement::Forbidden {
        return ReplayDecision::Blocked(BlockReason::Forbidden);
    }
    match envelope.completion {
        CompletionState::Partial => return ReplayDecision::Blocked(BlockReason::PartialState),
        CompletionState::Compacted => return ReplayDecision::Blocked(BlockReason::CompactedState),
        CompletionState::Complete => {}
    }
    if envelope.source.route_id != target.route_id
        || envelope.source.model_id != target.model_id
        || envelope.source.credential_scope != target.credential_scope
        || envelope.source.reasoning_mode != target.reasoning_mode
    {
        return ReplayDecision::Blocked(BlockReason::ProvenanceMismatch);
    }
    if policy.contract_id != Some(envelope.contract_id)
        || !state_matches_contract(envelope.contract_id, &envelope.continuation)
    {
        return ReplayDecision::Blocked(BlockReason::ContractMismatch);
    }
    if envelope.validate().is_err() {
        return ReplayDecision::Blocked(BlockReason::InvalidEnvelope);
    }
    if policy.activation != ActivationState::LiveValidated {
        return ReplayDecision::Blocked(BlockReason::NotLiveValidated);
    }
    ReplayDecision::Allowed
}

pub(crate) fn state_matches_contract(contract: ContractId, state: &ContinuationState) -> bool {
    matches!(
        (contract, state),
        (
            ContractId::OllamaNativeV1,
            ContinuationState::OllamaNative { .. }
        ) | (
            ContractId::GeminiCompatV1,
            ContinuationState::GeminiParts { .. }
        ) | (
            ContractId::MistralChunksV1,
            ContinuationState::MistralChunks { .. }
        ) | (
            ContractId::CerebrasChatV1,
            ContinuationState::CerebrasReasoning { .. }
        ) | (
            ContractId::OpenRouterDetailsV1,
            ContinuationState::OpenRouterDetails { .. }
        ) | (
            ContractId::DeepSeekChatV1,
            ContinuationState::ChatReasoning { .. }
        ) | (
            ContractId::KimiChatV1,
            ContinuationState::ChatReasoning { .. }
        ) | (
            ContractId::ZaiChatV1,
            ContinuationState::ChatReasoning { .. }
        ) | (
            ContractId::OpenAiResponsesV1,
            ContinuationState::ResponsesLocal { .. }
        ) | (
            ContractId::XaiResponsesV1,
            ContinuationState::ResponsesLocal { .. }
        ) | (
            ContractId::CodexResponsesV1,
            ContinuationState::ResponsesLocal { .. }
        )
    )
}
