#![allow(
    dead_code,
    reason = "the shared compression orchestrator consumes this staged candidate in Task 11"
)]

use super::checkpoint_document::CheckpointSection;
use super::checkpoint_selection::{CheckpointSelection, CheckpointSelectionLimits};
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
    pub report: CompressionSelectionReport,
}

pub async fn build(
    snapshot: &CompressionSnapshot,
    summary: Option<&ValidatedSummary>,
    sections: &[CheckpointSection],
) -> Result<CompressionCandidate, super::checkpoint_transaction::CompressionError> {
    validate_snapshot(snapshot)?;
    let band = resolved_band(snapshot)?;
    let selection = super::checkpoint_selection::select(
        &snapshot.source_messages,
        selection_limits(snapshot, band),
    )
    .map_err(super::checkpoint_transaction::CompressionError::from_code)?;
    let persisted_messages = super::checkpoint_document::assemble(
        &selection.messages,
        summary.map(|value| value.content.as_str()),
        sections,
        snapshot.trigger,
    )
    .map_err(super::checkpoint_transaction::CompressionError::from_code)?;
    let runtime_messages =
        super::checkpoint_candidate_runtime::project(snapshot, &persisted_messages);
    let after_tokens = super::token_estimate::estimate_request_tokens_for_provider(
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
        report,
    })
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

fn selection_limits(
    snapshot: &CompressionSnapshot,
    band: &CompressionBandSettings,
) -> CheckpointSelectionLimits {
    let window = budget_window(snapshot);
    let evidence = super::profile_budget::resolve_budget(&band.evidence_envelope, window);
    CheckpointSelectionLimits {
        user_tokens: category_tokens(&band.user_messages, window),
        assistant_tokens: category_tokens(&band.assistant_messages, window),
        tool_tokens: if band.tools.total_tokens > 0 {
            band.tools.total_tokens
        } else {
            evidence
        },
        tool_tokens_per_result: band.tools.tokens_per_item,
        max_tool_events: band.tools.max_items,
        total_tokens: target_tokens(window, band).saturating_sub(reserve_tokens(window, band)),
    }
}

fn validate_reduction(
    snapshot: &CompressionSnapshot,
    band: &CompressionBandSettings,
    selection: &CheckpointSelection,
    after_tokens: u32,
) -> Result<CompressionSelectionReport, super::checkpoint_transaction::CompressionError> {
    let known_window = (snapshot.context_window > 0).then_some(snapshot.context_window);
    let target = known_window.map(|window| target_tokens(window, band));
    let reserve = known_window.map(|window| reserve_tokens(window, band));
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

fn budget_window(snapshot: &CompressionSnapshot) -> u64 {
    snapshot
        .context_window
        .max(u64::from(snapshot.before_tokens).max(32_000))
}

fn category_tokens(budget: &super::profile_types::CategoryBudget, window: u64) -> u32 {
    if budget.enabled {
        super::profile_budget::resolve_budget(&budget.tokens, window)
    } else {
        0
    }
}

fn target_tokens(window: u64, band: &CompressionBandSettings) -> u32 {
    ((window as u128 * u128::from(band.target_percent)) / 100).min(u128::from(u32::MAX)) as u32
}

fn reserve_tokens(window: u64, band: &CompressionBandSettings) -> u32 {
    super::profile_budget::resolve_budget(&band.response_reserve, window)
}

pub(crate) fn same_messages(left: &[AgentMessage], right: &[AgentMessage]) -> bool {
    serde_json::to_vec(left).ok() == serde_json::to_vec(right).ok()
}
