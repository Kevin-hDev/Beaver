use crate::services::reasoning_continuity::contract::ContinuationTarget;
use crate::services::reasoning_continuity::envelope::ReasoningSource;
use crate::services::reasoning_continuity::registry::{ActivationState, ReplayRequirement};

pub(super) fn source_for_admission(target: &ContinuationTarget) -> Option<ReasoningSource> {
    let replay = target.replay()?;
    let policy = crate::services::reasoning_continuity::registry::replay_policy(replay)?;
    let activation_allowed = policy.activation() == ActivationState::LiveValidated
        || cfg!(debug_assertions) && target.is_fixture_candidate();
    (activation_allowed && policy.requirement() != ReplayRequirement::Forbidden)
        .then(|| ReasoningSource::from_target(replay))
}
