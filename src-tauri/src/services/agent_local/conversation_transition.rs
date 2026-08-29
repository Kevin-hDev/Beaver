use crate::services::reasoning_continuity::contract::{
    ContinuationTarget, ContinuationUse, ReplayTarget,
};
use crate::services::reasoning_continuity::envelope::{CompletionState, ReasoningSource};
use crate::services::reasoning_continuity::registry::ReplayRequirement;

use super::types_session::AgentSession;

/// A boundary is recorded instead of trying to translate native provider state.
/// The previous visible conversation remains available, but replay begins after it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContinuityBarrier {
    Model,
    Route,
    Credential,
    Mode,
    Fallback,
    Compaction,
    Legacy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transition {
    pub barrier: Option<ContinuityBarrier>,
    pub compatible_suffix_start: usize,
    pub replayable_message_indexes: Vec<usize>,
}

#[cfg(test)]
pub fn for_target(session: &AgentSession, target: &ReplayTarget) -> Transition {
    for_continuation(session, &ContinuationTarget::Replay(target.clone()))
}

pub fn for_continuation(session: &AgentSession, target: &ContinuationTarget) -> Transition {
    let mut result = Transition {
        barrier: None,
        compatible_suffix_start: 0,
        replayable_message_indexes: Vec::new(),
    };
    let Some(replay_target) = target.replay() else {
        return result;
    };
    let mut turn_start = 0usize;
    while turn_start < session.messages.len() {
        let turn_end = session.messages[turn_start..]
            .iter()
            .position(|message| message.turn_id != session.messages[turn_start].turn_id)
            .map_or(session.messages.len(), |offset| turn_start + offset);
        let compaction_boundary = session.messages[turn_start..turn_end]
            .iter()
            .any(|message| {
                crate::services::compress::state_recent::is_compression_context(&message.content)
            });
        let barrier = session.messages[turn_start..turn_end]
            .iter()
            .find_map(|message| {
                message
                    .continuation
                    .as_ref()
                    .map(|envelope| &envelope.source)
                    .or(message.replay_source.as_ref())
                    .and_then(|source| barrier_for(source, replay_target))
            });
        let has_provenance = session.messages[turn_start..turn_end]
            .iter()
            .any(|message| message.continuation.is_some() || message.replay_source.is_some());
        if compaction_boundary {
            result.barrier = Some(ContinuityBarrier::Compaction);
            result.compatible_suffix_start = turn_end;
            result.replayable_message_indexes.clear();
        } else if !has_provenance {
            // Les tours antérieurs au format v2 restent visibles, mais ne peuvent
            // pas satisfaire un contrat de rejeu natif qu'ils ne connaissaient pas.
            result.barrier = Some(ContinuityBarrier::Legacy);
            result.compatible_suffix_start = turn_end;
            result.replayable_message_indexes.clear();
        } else if let Some(barrier) = barrier {
            result.barrier = Some(barrier);
            // A provider state cannot be split from the visible user/assistant/tool turn it belongs to.
            result.compatible_suffix_start = turn_end;
            result.replayable_message_indexes.clear();
        } else if requires_fallback(target, &session.messages[turn_start..turn_end]) {
            // A completed visible turn without its required native state cannot
            // be replayed. Keep it visible and resume from the next turn.
            result.barrier = Some(ContinuityBarrier::Fallback);
            result.compatible_suffix_start = turn_end;
            result.replayable_message_indexes.clear();
        } else {
            for (offset, message) in session.messages[turn_start..turn_end].iter().enumerate() {
                if message.continuation.as_ref().is_some_and(|envelope| {
                    envelope.completion == CompletionState::Complete
                        && allows_replay(target, envelope)
                }) {
                    result.replayable_message_indexes.push(turn_start + offset);
                }
            }
        }
        turn_start = turn_end;
    }
    result
}

fn requires_fallback(
    target: &ContinuationTarget,
    turn: &[super::types_message::AgentMessage],
) -> bool {
    let Some(replay_target) = target.replay() else {
        return false;
    };
    let required_user_replay = replay_target.continuation_use == ContinuationUse::UserContinuation
        && crate::services::reasoning_continuity::registry::replay_policy(replay_target)
            .is_some_and(|policy| policy.requirement() == ReplayRequirement::Required);
    required_user_replay
        && turn.iter().any(|message| message.role == "assistant")
        && !turn
            .iter()
            .filter(|message| message.role == "assistant")
            .any(|message| {
                message.continuation.as_ref().is_some_and(|envelope| {
                    envelope.completion == CompletionState::Complete
                        && allows_replay(target, envelope)
                })
            })
}

fn allows_replay(
    target: &ContinuationTarget,
    envelope: &crate::services::reasoning_continuity::envelope::ReasoningEnvelope,
) -> bool {
    let Some(replay_target) = target.replay() else {
        return false;
    };
    #[cfg(debug_assertions)]
    if target.is_fixture_candidate() {
        return crate::services::reasoning_continuity::eligibility::decide_fixture_candidate(
            envelope,
            replay_target,
        ) == crate::services::reasoning_continuity::eligibility::ReplayDecision::Allowed;
    }
    crate::services::reasoning_continuity::eligibility::decide(envelope, replay_target)
        == crate::services::reasoning_continuity::eligibility::ReplayDecision::Allowed
}

fn barrier_for(source: &ReasoningSource, target: &ReplayTarget) -> Option<ContinuityBarrier> {
    if source.route_id != target.route_id {
        return Some(ContinuityBarrier::Route);
    }
    if source.model_id != target.model_id {
        return Some(ContinuityBarrier::Model);
    }
    if source.credential_scope != target.credential_scope {
        return Some(ContinuityBarrier::Credential);
    }
    (source.reasoning_mode != target.reasoning_mode).then_some(ContinuityBarrier::Mode)
}
