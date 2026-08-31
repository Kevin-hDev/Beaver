use std::path::Path;

use super::checkpoint_transaction::CompressionError;
use super::snapshot::CompressionSnapshot;
use super::summary_contract::ValidatedSummary;
use crate::services::agent_local::types_ollama::ChatMessage;

pub async fn build(
    snapshot: &CompressionSnapshot,
    summary: Option<&ValidatedSummary>,
    runtime: &[ChatMessage],
    working_dir: &Path,
) -> Result<super::checkpoint_candidate::CompressionCandidate, CompressionError> {
    let summary = summary.ok_or(CompressionError::SummaryInvalid)?;
    let collected = super::orchestrator_sections::collect(
        snapshot,
        runtime,
        working_dir,
        summary.estimated_tokens,
    )
    .await?;
    super::checkpoint_candidate::build_with_evidence(
        snapshot,
        Some(summary),
        &collected.sections,
        collected.evidence_tokens,
    )
    .await
}
