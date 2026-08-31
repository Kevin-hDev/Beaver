use std::path::Path;

use super::checkpoint_document::CheckpointSection;
use super::checkpoint_section_writer::SectionWriter;
use super::profile_types::{CompressionBandSettings, CompressionWindowBand};
use super::snapshot::CompressionSnapshot;

pub struct CollectedSections {
    pub sections: Vec<CheckpointSection>,
    pub evidence_tokens: u32,
}

pub async fn collect(
    snapshot: &CompressionSnapshot,
    runtime: &[crate::services::agent_local::types_ollama::ChatMessage],
    working_dir: &Path,
    summary_tokens: u32,
) -> Result<CollectedSections, super::checkpoint_transaction::CompressionError> {
    let (kind, settings) = band(snapshot);
    let envelope = super::checkpoint_evidence::envelope_tokens(kind).min(
        super::checkpoint_candidate_budget::available_after_summary(snapshot, kind, summary_tokens),
    );
    let mut writer = SectionWriter::new(envelope);
    if settings.include_work_state {
        push_work_state(snapshot, &mut writer).await?;
    }
    if snapshot.capabilities.project_context && settings.recent_file_count > 0 {
        let limit =
            super::checkpoint_evidence::item_limit(settings.recent_file_count, writer.remaining);
        let files = super::checkpoint_files::collect_with_budget(runtime, working_dir, limit).await;
        writer.push_evidence("files", &files, writer.remaining)?;
    }
    if writer.remaining > 0 {
        let limit = super::checkpoint_evidence::item_limit(8, writer.remaining);
        let attachments =
            super::checkpoint_text_attachments::collect(&snapshot.source_messages, limit).await;
        writer.push_evidence("text_attachments", &attachments, writer.remaining)?;
    }
    if writer.remaining > 0 {
        let limit = super::checkpoint_evidence::item_limit(32, writer.remaining);
        let references =
            super::checkpoint_reference_collect::collect(&snapshot.source_messages, limit);
        writer.push_evidence("critical_references", &references, writer.remaining)?;
    }
    Ok(CollectedSections {
        sections: writer.sections,
        evidence_tokens: envelope.saturating_sub(writer.remaining),
    })
}

async fn push_work_state(
    snapshot: &CompressionSnapshot,
    writer: &mut SectionWriter,
) -> Result<(), super::checkpoint_transaction::CompressionError> {
    let live =
        super::checkpoint_live_state::collect(&snapshot.source_session, &snapshot.capabilities);
    let share = (writer.remaining / 4).max(1);
    if snapshot.capabilities.git {
        writer.push_evidence("git", &(live.git, live.git_unavailable), share)?;
    }
    if snapshot.capabilities.plan_and_tasks {
        writer.push_evidence("plan_and_tasks", &(live.todos, live.active_plan), share)?;
    }
    writer.push_evidence("unresolved_state", &live.failures, share)?;
    if snapshot.capabilities.subagents {
        let subagents =
            super::checkpoint_subagents::collect(&snapshot.source_session, writer.remaining).await;
        writer.push_required("subagents", &subagents, writer.remaining)?;
    }
    Ok(())
}

fn band(snapshot: &CompressionSnapshot) -> (CompressionWindowBand, &CompressionBandSettings) {
    let kind = snapshot
        .profile
        .band(snapshot.context_window)
        .unwrap_or(CompressionWindowBand::Compact);
    (kind, snapshot.profile.profile.band_settings(kind))
}
