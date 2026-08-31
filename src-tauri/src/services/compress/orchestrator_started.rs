use super::checkpoint_transaction::{CompressionCommitReport, CompressionError};

pub(super) struct StartedCompression {
    pub report: CompressionCommitReport,
    pub facts: super::metrics::CompressionSuccessFacts,
}

pub(super) async fn run(
    request: super::orchestrator::CompressionRunRequest<'_>,
    session: crate::services::agent_local::types_session::AgentSession,
    profile: super::profile_resolve::ResolvedCompressionProfile,
    used_tokens: usize,
    attempt: Option<crate::services::agent_local::types_session::AutomaticCompressionAttempt>,
) -> Result<StartedCompression, CompressionError> {
    if request.cancel.is_cancelled() {
        return Err(CompressionError::Cancelled);
    }
    let tool_names = super::orchestrator_support::tool_names(request.provider_tools);
    let capabilities = super::session_capabilities::SessionCompressionCapabilities::from_runtime(
        request.chatbot,
        &tool_names,
        !session.working_dir.is_empty(),
        super::orchestrator_support::is_git_repository(request.working_dir),
        request.plan_mode_active,
    )
    .map_err(|_| CompressionError::SnapshotInvalid)?;
    let canonical = request
        .runtime_messages
        .iter()
        .filter(|message| message.role == "system")
        .cloned()
        .collect();
    let snapshot = super::snapshot::CompressionSnapshot::capture(
        &session,
        profile,
        request.context_window,
        capabilities,
        request.trigger,
    )
    .map_err(|_| CompressionError::SnapshotInvalid)?
    .with_runtime_context(
        canonical,
        request.provider_tools.to_vec(),
        used_tokens.min(u32::MAX as usize) as u32,
    )
    .map_err(|_| CompressionError::SnapshotInvalid)?;
    let images = collect_images(&snapshot);
    let snapshot = snapshot
        .with_checkpoint_images(images)
        .map_err(|_| CompressionError::SnapshotInvalid)?;
    let collector = super::orchestrator_summary::ProviderSummaryCollector {
        session_id: request.session_id,
        request_id: request.request_id,
        fast_mode: request.fast_mode,
        cancel: request.cancel,
    };
    let summary = super::orchestrator_summary::generate(&snapshot, &collector).await?;
    let mut candidate = super::orchestrator_candidate::build(
        &snapshot,
        summary.as_ref(),
        request.runtime_messages,
        request.working_dir,
    )
    .await?;
    candidate.automatic_compression_guard = super::automatic_guard::success_guard(
        attempt,
        candidate.after_tokens,
        request.context_window,
        snapshot.profile.profile.threshold_percent,
    );
    let next_count = snapshot.source_session.compression_count.saturating_add(1);
    let facts = super::metrics_facts::collect(&snapshot, summary.as_ref(), &candidate, next_count);
    let report = super::checkpoint_transaction::commit_candidate(
        request.session_id,
        request.runtime_messages,
        candidate,
    )
    .await?;
    Ok(StartedCompression { report, facts })
}

fn collect_images(
    snapshot: &super::snapshot::CompressionSnapshot,
) -> Vec<super::checkpoint_attachments::CheckpointImage> {
    let image_count = super::orchestrator_support::image_count(snapshot);
    if image_count == 0 {
        return Vec::new();
    }
    super::checkpoint_attachments::collect_images_with_limits(
        &snapshot.source_messages,
        super::checkpoint_attachments::MAX_IMAGE_CANDIDATES,
        32 * 1024 * 1024,
        usize::from(image_count),
    )
}
