use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::reasoning_continuity::contract::{ContinuationUse, ReplayTarget};
use crate::services::reasoning_continuity::eligibility::{self, ReplayDecision};
use crate::services::reasoning_continuity::envelope::ReasoningEnvelope;
use crate::services::reasoning_continuity::registry::{AdapterId, ReplayPolicy};

#[cfg(test)]
use crate::services::reasoning_continuity::contract::ContractId;
#[cfg(test)]
use crate::services::reasoning_continuity::envelope::ContinuationState;

#[path = "replay_apply.rs"]
mod replay_apply;
#[cfg(test)]
pub(crate) use replay_apply::apply_chat_continuity;
pub(crate) use replay_apply::{
    apply_chat_continuity_at, apply_chat_payload_continuity, apply_ollama_continuity,
    apply_responses_continuity,
};

#[cfg(debug_assertions)]
pub(crate) mod fixture_candidate;

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

/// Le type de cible est conservé jusqu'au transport : une fixture debug ne
/// peut contourner que l'activation, jamais la provenance ni le contrat.
pub(crate) fn approval_for_target<'a>(
    target: &'a crate::services::reasoning_continuity::contract::ContinuationTarget,
    envelope: &'a ReasoningEnvelope,
) -> Result<ReplayApproval<'a>, ReplayApplyError> {
    let replay_target = target.replay().ok_or(ReplayApplyError::Blocked)?;
    let policy = crate::services::reasoning_continuity::registry::replay_policy(replay_target)
        .ok_or(ReplayApplyError::Blocked)?;
    #[cfg(debug_assertions)]
    if target.is_fixture_candidate() {
        return fixture_candidate::approved(policy, envelope, replay_target);
    }
    approved(
        eligibility::decide(envelope, replay_target),
        policy,
        envelope,
        replay_target,
    )
}

/// Le dernier message détermine le type de continuité de l'appel sortant.
/// Cette décision tardive conserve la provenance admise au premier tour.
pub(crate) fn target_for_request(
    messages: &[ChatMessage],
    target: Option<&crate::services::reasoning_continuity::contract::ContinuationTarget>,
) -> Option<crate::services::reasoning_continuity::contract::ContinuationTarget> {
    let continuation_use = if messages
        .last()
        .is_some_and(|message| message.role == "tool")
    {
        ContinuationUse::ToolContinuation
    } else {
        ContinuationUse::UserContinuation
    };
    target.map(|target| target.for_continuation_use(continuation_use))
}

#[cfg(test)]
#[path = "replay_tests.rs"]
mod tests;
