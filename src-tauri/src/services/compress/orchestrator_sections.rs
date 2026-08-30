use std::path::Path;

use super::checkpoint_document::CheckpointSection;
use super::profile_types::{CompressionBandSettings, CompressionWindowBand};
use super::snapshot::CompressionSnapshot;

pub async fn collect(
    snapshot: &CompressionSnapshot,
    runtime: &[crate::services::agent_local::types_ollama::ChatMessage],
    working_dir: &Path,
) -> Result<Vec<CheckpointSection>, super::checkpoint_transaction::CompressionError> {
    let band = band(snapshot);
    let mut sections = Vec::new();
    let live =
        super::checkpoint_live_state::collect(&snapshot.source_session, &snapshot.capabilities);
    push_json(&mut sections, "live_state", &live)?;
    if snapshot.capabilities.project_context && band.files.enabled {
        let files =
            super::checkpoint_files::collect(runtime, working_dir, snapshot.context_window).await;
        push_json(&mut sections, "files", &files)?;
    }
    if snapshot.capabilities.subagents && band.subagent_detail_tokens.enabled {
        let tokens = super::profile_budget::resolve_budget(
            &band.subagent_detail_tokens.tokens,
            budget_window(snapshot),
        );
        let subagents =
            super::checkpoint_subagents::collect(&snapshot.source_session, tokens).await;
        push_json(&mut sections, "subagents", &subagents)?;
    }
    Ok(sections)
}

fn push_json<T: serde::Serialize>(
    output: &mut Vec<CheckpointSection>,
    name: &str,
    value: &T,
) -> Result<(), super::checkpoint_transaction::CompressionError> {
    let content = serde_json::to_string(value)
        .map_err(|_| super::checkpoint_transaction::CompressionError::CandidateInvalid)?;
    output.push(CheckpointSection {
        name: name.to_string(),
        content,
    });
    Ok(())
}

fn band(snapshot: &CompressionSnapshot) -> &CompressionBandSettings {
    match snapshot.profile.band(snapshot.context_window) {
        Some(CompressionWindowBand::Under64K) => &snapshot.profile.profile.under_64k,
        Some(CompressionWindowBand::Large) => &snapshot.profile.profile.large,
        Some(CompressionWindowBand::Compact) | None => &snapshot.profile.profile.compact,
    }
}

fn budget_window(snapshot: &CompressionSnapshot) -> u64 {
    snapshot
        .context_window
        .max(u64::from(snapshot.before_tokens).max(32_000))
}
