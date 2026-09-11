use super::checkpoint_candidate::CompressionSelectionReport;
use super::checkpoint_selection::CheckpointSelection;
use super::profile_types::{CompressionBandSettings, CompressionTrigger, CompressionWindowBand};
use super::snapshot::CompressionSnapshot;
use crate::services::agent_local::types_message::AgentMessage;

pub(super) fn validate_snapshot(
    snapshot: &CompressionSnapshot,
) -> Result<(), super::checkpoint_transaction::CompressionError> {
    if snapshot.source_session.id != snapshot.session_id
        || !same_messages(&snapshot.source_session.messages, &snapshot.source_messages)
        || snapshot.source_messages.is_empty()
    {
        return Err(super::checkpoint_transaction::CompressionError::SnapshotInvalid);
    }
    if !snapshot.capacity_verified() {
        return Err(super::checkpoint_transaction::CompressionError::CapacityUnverified);
    }
    crate::services::agent_local::conversation_history_validation::validate(
        &snapshot.source_messages,
    )
    .map(|_| ())
    .map_err(|_| super::checkpoint_transaction::CompressionError::OpenTurn)
}

pub(super) fn resolved_band(
    snapshot: &CompressionSnapshot,
) -> Result<
    (CompressionWindowBand, &CompressionBandSettings),
    super::checkpoint_transaction::CompressionError,
> {
    match snapshot.profile.band(snapshot.context_window) {
        Some(CompressionWindowBand::Under64K) if snapshot.profile.profile.allow_under_64k => Ok((
            CompressionWindowBand::Under64K,
            &snapshot.profile.profile.under_64k,
        )),
        Some(CompressionWindowBand::Under64K) => {
            Err(super::checkpoint_transaction::CompressionError::Unavailable)
        }
        Some(CompressionWindowBand::Compact) => Ok((
            CompressionWindowBand::Compact,
            &snapshot.profile.profile.compact,
        )),
        Some(CompressionWindowBand::Large) => Ok((
            CompressionWindowBand::Large,
            &snapshot.profile.profile.large,
        )),
        None if snapshot.trigger == CompressionTrigger::Explicit => Ok((
            CompressionWindowBand::Compact,
            &snapshot.profile.profile.compact,
        )),
        None => Err(super::checkpoint_transaction::CompressionError::Unavailable),
    }
}

pub(super) fn validate_reduction(
    snapshot: &CompressionSnapshot,
    kind: CompressionWindowBand,
    selection: &CheckpointSelection,
    after_tokens: u32,
) -> Result<CompressionSelectionReport, super::checkpoint_transaction::CompressionError> {
    let target = super::checkpoint_candidate_budget::target_tokens(snapshot, kind);
    let checkpoint_tokens = after_tokens.saturating_sub(selection.active_turn_tokens);
    if checkpoint_tokens > target {
        return Err(super::checkpoint_transaction::CompressionError::CapacityExceeded);
    }
    let compressible_after = checkpoint_tokens.saturating_sub(snapshot.system_head_tokens());
    if snapshot.trigger == CompressionTrigger::Automatic
        && super::token_estimate::should_compress(
            compressible_after as usize,
            snapshot.context_window,
            snapshot.profile.profile.threshold_percent,
        )
    {
        return Err(super::checkpoint_transaction::CompressionError::InsufficientReduction);
    }
    Ok(CompressionSelectionReport {
        selected_messages: selection.messages.len(),
        before_tokens: snapshot.before_tokens(),
        after_tokens,
        target_tokens: Some(target),
        reserve_tokens: None,
        minimum_reduction_tokens: None,
    })
}

pub(crate) fn same_messages(left: &[AgentMessage], right: &[AgentMessage]) -> bool {
    match (serde_json::to_vec(left), serde_json::to_vec(right)) {
        (Ok(left), Ok(right)) => left == right,
        _ => false,
    }
}
