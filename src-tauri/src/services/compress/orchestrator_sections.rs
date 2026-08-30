use std::path::Path;

use super::checkpoint_document::CheckpointSection;
use super::checkpoint_section_writer::SectionWriter;
use super::profile_types::{CompressionBandSettings, CompressionWindowBand, ItemBudget};
use super::snapshot::CompressionSnapshot;

pub struct CollectedSections {
    pub sections: Vec<CheckpointSection>,
    pub evidence_tokens: u32,
}

pub async fn collect(
    snapshot: &CompressionSnapshot,
    runtime: &[crate::services::agent_local::types_ollama::ChatMessage],
    working_dir: &Path,
) -> Result<CollectedSections, super::checkpoint_transaction::CompressionError> {
    let band = band(snapshot);
    let window = budget_window(snapshot);
    let envelope = super::profile_budget::resolve_budget(&band.evidence_envelope, window);
    let mut writer = SectionWriter::new(envelope);
    let live =
        super::checkpoint_live_state::collect(&snapshot.source_session, &snapshot.capabilities);

    if snapshot.capabilities.git && band.git_tokens.enabled {
        writer.push_evidence(
            "git",
            &(live.git, live.git_unavailable),
            super::profile_budget::resolve_budget(&band.git_tokens.tokens, window),
        )?;
    }
    if snapshot.capabilities.plan_and_tasks && band.plan_and_tasks_tokens.enabled {
        writer.push_evidence(
            "plan_and_tasks",
            &(live.todos, live.active_plan),
            super::profile_budget::resolve_budget(&band.plan_and_tasks_tokens.tokens, window),
        )?;
    }
    if band.unresolved_state_tokens.enabled {
        writer.push_evidence(
            "unresolved_state",
            &live.failures,
            super::profile_budget::resolve_budget(&band.unresolved_state_tokens.tokens, window),
        )?;
    }
    if snapshot.capabilities.project_context && band.files.enabled {
        let files =
            super::checkpoint_files::collect_read_with_budget(runtime, working_dir, &band.files)
                .await;
        writer.push_evidence("files", &files, item_limit(&band.files, writer.remaining))?;
    }
    if snapshot.capabilities.project_context && band.modified_files.enabled {
        let files = super::checkpoint_files::collect_modified_with_budget(
            runtime,
            working_dir,
            &band.modified_files,
        )
        .await;
        writer.push_evidence(
            "modified_files",
            &files,
            item_limit(&band.modified_files, writer.remaining),
        )?;
    }
    if band.text_attachments.enabled {
        let attachments = super::checkpoint_text_attachments::collect(
            &snapshot.source_messages,
            &band.text_attachments,
        )
        .await;
        writer.push_evidence(
            "text_attachments",
            &attachments,
            item_limit(&band.text_attachments, writer.remaining),
        )?;
    }
    if band.critical_references.enabled {
        let references = super::checkpoint_reference_collect::collect(
            &snapshot.source_messages,
            &band.critical_references,
        );
        writer.push_evidence(
            "critical_references",
            &references,
            item_limit(&band.critical_references, writer.remaining),
        )?;
    }
    if snapshot.capabilities.subagents && band.subagent_detail_tokens.enabled {
        let tokens =
            super::profile_budget::resolve_budget(&band.subagent_detail_tokens.tokens, window);
        let subagents =
            super::checkpoint_subagents::collect(&snapshot.source_session, tokens).await;
        writer.push_independent("subagents", &subagents, tokens)?;
    }
    Ok(CollectedSections {
        sections: writer.sections,
        evidence_tokens: envelope.saturating_sub(writer.remaining),
    })
}

fn item_limit(budget: &ItemBudget, remaining: u32) -> u32 {
    if budget.total_tokens == 0 {
        remaining
    } else {
        budget.total_tokens
    }
}

fn band(snapshot: &CompressionSnapshot) -> &CompressionBandSettings {
    match snapshot.profile.band(snapshot.context_window) {
        Some(CompressionWindowBand::Under64K) => &snapshot.profile.profile.under_64k,
        Some(CompressionWindowBand::Large) => &snapshot.profile.profile.large,
        Some(CompressionWindowBand::Compact) | None => &snapshot.profile.profile.compact,
    }
}

fn budget_window(snapshot: &CompressionSnapshot) -> u64 {
    super::profile_budget::effective_budget_window(snapshot.context_window, snapshot.before_tokens)
}
