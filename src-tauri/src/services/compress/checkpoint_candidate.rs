#![allow(
    dead_code,
    reason = "the shared compression orchestrator consumes this staged candidate in Task 11"
)]

use super::checkpoint_document::CheckpointSection;
use super::checkpoint_selection::CheckpointSelection;
use super::profile_types::{CompressionBandSettings, CompressionWindowBand};
use super::snapshot::CompressionSnapshot;
use super::summary_contract::ValidatedSummary;
use crate::services::agent_local::types_message::AgentMessage;
use crate::services::agent_local::types_ollama::ChatMessage;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompressionSelectionReport {
    pub selected_messages: usize,
    pub before_tokens: u32,
    pub after_tokens: u32,
    pub target_tokens: Option<u32>,
    pub reserve_tokens: Option<u32>,
    pub minimum_reduction_tokens: Option<u32>,
}

pub struct CompressionCandidate {
    pub source_messages: Vec<AgentMessage>,
    pub persisted_messages: Vec<AgentMessage>,
    pub runtime_messages: Vec<ChatMessage>,
    pub before_tokens: u32,
    pub after_tokens: u32,
    pub retained_images: usize,
    pub report: CompressionSelectionReport,
}

pub async fn build(
    snapshot: &CompressionSnapshot,
    summary: Option<&ValidatedSummary>,
    sections: &[CheckpointSection],
) -> Result<CompressionCandidate, super::checkpoint_transaction::CompressionError> {
    build_with_evidence(snapshot, summary, sections, 0).await
}

pub async fn build_with_evidence(
    snapshot: &CompressionSnapshot,
    summary: Option<&ValidatedSummary>,
    sections: &[CheckpointSection],
    evidence_tokens: u32,
) -> Result<CompressionCandidate, super::checkpoint_transaction::CompressionError> {
    validate_snapshot(snapshot)?;
    let band = resolved_band(snapshot)?;
    let selection = super::checkpoint_selection::select(
        &snapshot.source_messages,
        super::checkpoint_candidate_budget::selection_limits(
            snapshot,
            band,
            summary,
            sections,
            evidence_tokens,
        ),
    )
    .map_err(super::checkpoint_transaction::CompressionError::from_code)?;
    let mut persisted_messages = super::checkpoint_document::assemble(
        &selection.messages,
        active_turn_id(snapshot, &selection),
        summary.map(|value| value.content.as_str()),
        sections,
        snapshot.trigger,
    )
    .map_err(super::checkpoint_transaction::CompressionError::from_code)?;
    let (retained_images, retained_source_message_ids) =
        super::checkpoint_candidate_images::prepare(
            snapshot,
            &selection,
            &persisted_messages,
            band,
        );
    let mut runtime_snapshot = snapshot.clone();
    runtime_snapshot.checkpoint_images = retained_images;
    let mut runtime_messages =
        super::checkpoint_candidate_runtime::project(&runtime_snapshot, &persisted_messages);
    let mut after_tokens = super::token_estimate::estimate_request_tokens_for_provider(
        &snapshot.provider_id,
        &runtime_messages,
        &snapshot.provider_tools,
    )
    .min(u32::MAX as usize) as u32;
    super::checkpoint_metadata::set(
        &mut persisted_messages,
        snapshot,
        after_tokens,
        sections,
        retained_source_message_ids,
    )
    .map_err(super::checkpoint_transaction::CompressionError::from_code)?;
    runtime_messages =
        super::checkpoint_candidate_runtime::project(&runtime_snapshot, &persisted_messages);
    after_tokens = super::token_estimate::estimate_request_tokens_for_provider(
        &snapshot.provider_id,
        &runtime_messages,
        &snapshot.provider_tools,
    )
    .min(u32::MAX as usize) as u32;
    let report = validate_reduction(snapshot, band, &selection, after_tokens)?;
    prepare_candidate(snapshot, &persisted_messages).await?;
    Ok(CompressionCandidate {
        source_messages: snapshot.source_messages.clone(),
        persisted_messages,
        runtime_messages,
        before_tokens: snapshot.before_tokens,
        after_tokens,
        retained_images: runtime_snapshot.checkpoint_images.len(),
        report,
    })
}

fn active_turn_id<'a>(
    snapshot: &'a CompressionSnapshot,
    selection: &CheckpointSelection,
) -> Option<&'a str> {
    selection
        .units
        .iter()
        .find(|unit| unit.kind == super::checkpoint_units::CheckpointUnitKind::ActiveTurn)
        .and_then(|unit| snapshot.source_messages.get(unit.message_indexes.start))
        .map(|message| message.turn_id.as_str())
}

async fn prepare_candidate(
    snapshot: &CompressionSnapshot,
    messages: &[AgentMessage],
) -> Result<(), super::checkpoint_transaction::CompressionError> {
    let mut prepared = snapshot.source_session.clone();
    prepared.messages = messages.to_vec();
    prepared.compression_count = prepared.compression_count.saturating_add(1);
    crate::services::agent_local::session_store_messages::recompute_accumulated_tokens(
        &mut prepared,
    );
    crate::services::agent_local::session_store::prepare_document(&prepared)
        .await
        .map(|_| ())
        .map_err(|_| super::checkpoint_transaction::CompressionError::PrepareFailed)
}

fn validate_snapshot(
    snapshot: &CompressionSnapshot,
) -> Result<(), super::checkpoint_transaction::CompressionError> {
    if snapshot.source_session.id != snapshot.session_id
        || !same_messages(&snapshot.source_session.messages, &snapshot.source_messages)
        || snapshot.source_messages.is_empty()
    {
        return Err(super::checkpoint_transaction::CompressionError::SnapshotInvalid);
    }
    crate::services::agent_local::conversation_history_validation::validate(
        &snapshot.source_messages,
    )
    .map_err(|_| super::checkpoint_transaction::CompressionError::OpenTurn)?;
    Ok(())
}

fn resolved_band(
    snapshot: &CompressionSnapshot,
) -> Result<&CompressionBandSettings, super::checkpoint_transaction::CompressionError> {
    match snapshot.profile.band(snapshot.context_window) {
        Some(CompressionWindowBand::Under64K) if snapshot.profile.profile.allow_under_64k => {
            Ok(&snapshot.profile.profile.under_64k)
        }
        Some(CompressionWindowBand::Under64K) => {
            Err(super::checkpoint_transaction::CompressionError::Unavailable)
        }
        Some(CompressionWindowBand::Compact) => Ok(&snapshot.profile.profile.compact),
        Some(CompressionWindowBand::Large) => Ok(&snapshot.profile.profile.large),
        None if snapshot.trigger == super::profile_types::CompressionTrigger::Explicit => {
            Ok(&snapshot.profile.profile.compact)
        }
        None => Err(super::checkpoint_transaction::CompressionError::Unavailable),
    }
}

fn validate_reduction(
    snapshot: &CompressionSnapshot,
    band: &CompressionBandSettings,
    selection: &CheckpointSelection,
    after_tokens: u32,
) -> Result<CompressionSelectionReport, super::checkpoint_transaction::CompressionError> {
    let known_window = (snapshot.context_window > 0).then_some(snapshot.context_window);
    let target =
        known_window.map(|window| super::checkpoint_candidate_budget::target_tokens(window, band));
    let reserve =
        known_window.map(|window| super::checkpoint_candidate_budget::reserve_tokens(window, band));
    let minimum = known_window
        .map(|window| super::profile_budget::resolve_budget(&band.minimum_reduction, window));
    if target
        .zip(reserve)
        .is_some_and(|(target, reserve)| after_tokens > target.saturating_sub(reserve))
    {
        return Err(super::checkpoint_transaction::CompressionError::CapacityExceeded);
    }
    if minimum.is_some_and(|minimum| snapshot.before_tokens.saturating_sub(after_tokens) < minimum)
    {
        return Err(super::checkpoint_transaction::CompressionError::InsufficientReduction);
    }
    Ok(CompressionSelectionReport {
        selected_messages: selection.messages.len(),
        before_tokens: snapshot.before_tokens,
        after_tokens,
        target_tokens: target,
        reserve_tokens: reserve,
        minimum_reduction_tokens: minimum,
    })
}

pub(crate) fn same_messages(left: &[AgentMessage], right: &[AgentMessage]) -> bool {
    match (serde_json::to_vec(left), serde_json::to_vec(right)) {
        (Ok(left), Ok(right)) => left == right,
        _ => false,
    }
}
