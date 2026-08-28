use super::{ReplayApplyError, ReplayApproval};
use crate::services::reasoning_continuity::contract::ReplayTarget;
use crate::services::reasoning_continuity::eligibility::{self, ReplayDecision};
use crate::services::reasoning_continuity::envelope::ReasoningEnvelope;
use crate::services::reasoning_continuity::registry::ReplayPolicy;

/// Construit une preuve de rejeu pour une fixture debug seulement. La décision
/// centrale conserve route, modèle, scope, contrat et interdictions ; seule
/// l'activation live est temporairement contournée pour produire la preuve.
pub(crate) fn approved<'a>(
    policy: ReplayPolicy,
    envelope: &'a ReasoningEnvelope,
    target: &'a ReplayTarget,
) -> Result<ReplayApproval<'a>, ReplayApplyError> {
    if eligibility::decide_fixture_candidate(envelope, target) != ReplayDecision::Allowed {
        return Err(ReplayApplyError::Blocked);
    }
    let Some((contract_id, adapter)) = policy.fixture_adapter() else {
        return Err(ReplayApplyError::Blocked);
    };
    if contract_id != envelope.contract_id || !envelope.source.matches_target(target) {
        return Err(ReplayApplyError::ContractMismatch);
    }
    Ok(ReplayApproval {
        envelope,
        target,
        adapter,
    })
}
