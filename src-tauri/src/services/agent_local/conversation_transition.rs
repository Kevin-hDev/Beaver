use crate::services::reasoning_continuity::contract::ReplayTarget;
use crate::services::reasoning_continuity::envelope::{CompletionState, ReasoningSource};

use super::types_session::AgentSession;

/// A boundary is recorded instead of trying to translate native provider state.
/// The previous visible conversation remains available, but replay begins after it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code, reason = "fallback and durable compaction adopt this closed barrier in later provider tasks")]
pub enum ContinuityBarrier {
    Model,
    Route,
    Credential,
    Mode,
    Fallback,
    Compaction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transition {
    pub barrier: Option<ContinuityBarrier>,
    pub compatible_suffix_start: usize,
    pub replayable_message_indexes: Vec<usize>,
}

pub fn for_target(session: &AgentSession, target: &ReplayTarget) -> Transition {
    let mut result = Transition {
        barrier: None,
        compatible_suffix_start: 0,
        replayable_message_indexes: Vec::new(),
    };
    let mut turn_start = 0usize;
    while turn_start < session.messages.len() {
        let turn_end = session.messages[turn_start..]
            .iter()
            .position(|message| message.turn_id != session.messages[turn_start].turn_id)
            .map_or(session.messages.len(), |offset| turn_start + offset);
        let barrier = session.messages[turn_start..turn_end]
            .iter()
            .find_map(|message| {
                message
                    .continuation
                    .as_ref()
                    .map(|envelope| &envelope.source)
                    .or(message.replay_source.as_ref())
                    .and_then(|source| barrier_for(source, target))
            });
        if let Some(barrier) = barrier {
            result.barrier = Some(barrier);
            // A provider state cannot be split from the visible user/assistant/tool turn it belongs to.
            result.compatible_suffix_start = turn_end;
            result.replayable_message_indexes.clear();
        } else {
            for (offset, message) in session.messages[turn_start..turn_end].iter().enumerate() {
                if message.continuation.as_ref().is_some_and(|envelope| {
                    envelope.completion == CompletionState::Complete
                        && crate::services::reasoning_continuity::eligibility::decide(envelope, target)
                            == crate::services::reasoning_continuity::eligibility::ReplayDecision::Allowed
                }) {
                    result.replayable_message_indexes.push(turn_start + offset);
                }
            }
        }
        turn_start = turn_end;
    }
    result
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
