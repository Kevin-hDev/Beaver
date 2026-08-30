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
    decide_with_activation(envelope, target, false)
}

/// Réservé aux fixtures debug : toutes les vérifications sauf l'état
/// `LiveValidated` restent obligatoires afin de prouver un couple exact avant
/// de l'activer. Ce point ne fait pas partie du binaire de production.
#[cfg(debug_assertions)]
pub(crate) fn decide_fixture_candidate(
    envelope: &ReasoningEnvelope,
    target: &ReplayTarget,
) -> ReplayDecision {
    decide_with_activation(envelope, target, true)
}

fn decide_with_activation(
    envelope: &ReasoningEnvelope,
    target: &ReplayTarget,
    allow_fixture_candidate: bool,
) -> ReplayDecision {
    if target.validate().is_err() {
        return ReplayDecision::Blocked(BlockReason::UnknownTarget);
    }
    let Some(policy) = registry::replay_policy(target) else {
        return ReplayDecision::Blocked(BlockReason::UnknownTarget);
    };
    if policy.requirement() == ReplayRequirement::Forbidden {
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
    if policy.fixture_adapter().map(|value| value.0) != Some(envelope.contract_id)
        || !state_matches_contract(envelope.contract_id, &envelope.continuation)
    {
        return ReplayDecision::Blocked(BlockReason::ContractMismatch);
    }
    if envelope.validate().is_err() {
        return ReplayDecision::Blocked(BlockReason::InvalidEnvelope);
    }
    if policy.activation() != ActivationState::LiveValidated && !allow_fixture_candidate {
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
            ContractId::AnthropicMessagesV1,
            ContinuationState::AnthropicBlocks { .. }
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
            ContractId::QwenChatV1,
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

#[cfg(all(test, debug_assertions))]
mod tests {
    use super::*;
    use crate::services::reasoning_continuity::contract::{
        ContinuationUse, CredentialScope, ReasoningModeId, RouteId,
    };
    use crate::services::reasoning_continuity::envelope::{ReasoningEnvelope, ReasoningSource};

    fn fixture_target(model_id: &str) -> ReplayTarget {
        ReplayTarget {
            route_id: RouteId::Ollama,
            model_id: model_id.into(),
            credential_scope: CredentialScope::local_uncredentialed(),
            reasoning_mode: ReasoningModeId::Auto,
            continuation_use: ContinuationUse::UserContinuation,
        }
    }

    fn fixture_envelope(target: &ReplayTarget) -> ReasoningEnvelope {
        ReasoningEnvelope::new(
            ContractId::OllamaNativeV1,
            ReasoningSource::from_target(target),
            CompletionState::Complete,
            ContinuationState::OllamaNative {
                thinking: "opaque fixture".into(),
            },
            Vec::new(),
        )
    }

    #[test]
    fn fixture_candidate_bypasses_only_activation_for_an_exact_policy() {
        let target = fixture_target("deepseek-r1:latest");
        let envelope = fixture_envelope(&target);
        assert_eq!(
            decide(&envelope, &target),
            ReplayDecision::Blocked(BlockReason::NotLiveValidated)
        );
        assert_eq!(
            decide_fixture_candidate(&envelope, &target),
            ReplayDecision::Allowed
        );

        let unknown = fixture_target("unknown-model");
        assert_eq!(
            decide_fixture_candidate(&fixture_envelope(&unknown), &unknown),
            ReplayDecision::Blocked(BlockReason::UnknownTarget)
        );
    }
}
