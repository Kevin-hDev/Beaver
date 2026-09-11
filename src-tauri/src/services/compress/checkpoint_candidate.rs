use super::checkpoint_document::CheckpointSection;
use super::checkpoint_selection::CheckpointSelection;
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
    pub prepared_count: crate::services::agent_local::context_usage_record::ContextTokenCount,
    pub retained_images: usize,
    pub report: CompressionSelectionReport,
    pub automatic_compression_guard:
        crate::services::agent_local::types_session::AutomaticCompressionGuard,
}

#[cfg(test)]
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
    super::checkpoint_candidate_validation::validate_snapshot(snapshot)?;
    let (kind, band) = super::checkpoint_candidate_validation::resolved_band(snapshot)?;
    let summary = summary.ok_or(super::checkpoint_transaction::CompressionError::SummaryInvalid)?;
    let selection = super::checkpoint_selection::select(
        &snapshot.source_messages,
        super::checkpoint_candidate_budget::selection_limits(
            snapshot,
            kind,
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
        Some(summary.content.as_str()),
        sections,
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
    let mut after_tokens = super::prepared_request::count(
        &snapshot.provider_id,
        &snapshot.source_session.model,
        &runtime_messages,
        &snapshot.provider_tools,
    )
    .capacity_tokens
    .ok_or(super::checkpoint_transaction::CompressionError::CapacityUnverified)?;
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
    let prepared_count = super::prepared_request::count(
        &snapshot.provider_id,
        &snapshot.source_session.model,
        &runtime_messages,
        &snapshot.provider_tools,
    );
    after_tokens = prepared_count
        .capacity_tokens
        .ok_or(super::checkpoint_transaction::CompressionError::CapacityUnverified)?;
    let report = super::checkpoint_candidate_validation::validate_reduction(
        snapshot,
        kind,
        &selection,
        after_tokens,
    )?;
    prepare_candidate(snapshot, &persisted_messages).await?;
    Ok(CompressionCandidate {
        source_messages: snapshot.source_messages.clone(),
        persisted_messages,
        runtime_messages,
        before_tokens: snapshot.before_tokens(),
        after_tokens,
        prepared_count,
        retained_images: runtime_snapshot.checkpoint_images.len(),
        report,
        automatic_compression_guard: snapshot.source_session.automatic_compression_guard.clone(),
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
