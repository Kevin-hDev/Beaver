use std::path::Path;

use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::{ChatMessage, StreamEvent};
use tokio_util::sync::CancellationToken;

use super::checkpoint_transaction::{CompressionCommitReport, CompressionError};
use super::profile_types::CompressionTrigger;

pub struct CompressionRunRequest<'a> {
    pub on_event: &'a AgentEventEmitter,
    pub session_id: &'a str,
    pub request_id: &'a str,
    pub trigger: CompressionTrigger,
    pub runtime_messages: &'a mut Vec<ChatMessage>,
    pub provider_id: &'a str,
    pub fast_mode: crate::services::llm::fast_mode::FastModeRequest,
    pub context_window: u64,
    pub last_context_tokens: Option<u32>,
    pub provider_tools: &'a [serde_json::Value],
    pub chatbot: bool,
    pub plan_mode_active: bool,
    pub working_dir: &'a Path,
    pub cancel: CancellationToken,
}

pub async fn run_compression(
    request: CompressionRunRequest<'_>,
) -> Result<Option<CompressionCommitReport>, CompressionError> {
    let session = crate::services::agent_local::session_store::get(request.session_id)
        .await
        .map_err(|_| CompressionError::SnapshotInvalid)?;
    let profile = super::profile_resolve::resolve_for_session(&session)
        .map_err(|_| CompressionError::Unavailable)?;
    let estimated = super::token_estimate::estimate_request_tokens_for_provider(
        request.provider_id,
        request.runtime_messages,
        request.provider_tools,
    );
    let used = super::state::context_used_for_compression(request.last_context_tokens, estimated);
    if !eligible(&profile, request.trigger, request.context_window, used) {
        return Ok(None);
    }
    let on_event = request.on_event;
    send(on_event, "start");
    crate::services::agent_local::stream_diagnostics::mark_phase(
        request.session_id,
        request.request_id,
        "compression",
        "Compression du contexte démarrée.",
    )
    .await;
    let result = run_started(request, session, profile, used).await;
    send(on_event, "done");
    if result.is_ok() {
        let _ = on_event.send(StreamEvent::CompressionComplete {});
    }
    result.map(Some)
}

async fn run_started(
    request: CompressionRunRequest<'_>,
    session: crate::services::agent_local::types_session::AgentSession,
    profile: super::profile_resolve::ResolvedCompressionProfile,
    used_tokens: usize,
) -> Result<CompressionCommitReport, CompressionError> {
    if request.cancel.is_cancelled() {
        return Err(CompressionError::SummaryInvalid);
    }
    let tool_names = tool_names(request.provider_tools);
    let capabilities = super::session_capabilities::SessionCompressionCapabilities::from_runtime(
        request.chatbot,
        &tool_names,
        !session.working_dir.is_empty(),
        is_git_repository(request.working_dir),
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
    let image_budget = image_budget(&snapshot);
    let images = if image_budget.enabled {
        super::checkpoint_attachments::collect_images_with_limits(
            &snapshot.source_messages,
            usize::from(image_budget.max_items),
            image_budget.max_total_bytes,
            16,
        )
    } else {
        Vec::new()
    };
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
    let candidate = super::orchestrator_candidate::build(
        &snapshot,
        summary.as_ref(),
        request.runtime_messages,
        request.working_dir,
    )
    .await?;
    super::checkpoint_transaction::commit_candidate(
        request.session_id,
        request.runtime_messages,
        candidate,
    )
    .await
}

fn image_budget(
    snapshot: &super::snapshot::CompressionSnapshot,
) -> super::profile_types::ImageBudget {
    match snapshot.profile.band(snapshot.context_window) {
        Some(super::profile_types::CompressionWindowBand::Under64K) => {
            snapshot.profile.profile.under_64k.images
        }
        Some(super::profile_types::CompressionWindowBand::Large) => {
            snapshot.profile.profile.large.images
        }
        Some(super::profile_types::CompressionWindowBand::Compact) | None => {
            snapshot.profile.profile.compact.images
        }
    }
}

pub(super) fn eligible(
    profile: &super::profile_resolve::ResolvedCompressionProfile,
    trigger: CompressionTrigger,
    context_window: u64,
    used_tokens: usize,
) -> bool {
    if !profile.available(context_window) {
        return false;
    }
    match trigger {
        CompressionTrigger::Explicit => true,
        CompressionTrigger::Automatic => {
            context_window > 0
                && super::token_estimate::should_compress(
                    used_tokens,
                    context_window,
                    profile.profile.threshold_percent.min(90),
                )
        }
    }
}

fn tool_names(tools: &[serde_json::Value]) -> Vec<String> {
    tools
        .iter()
        .filter_map(|tool| {
            tool.pointer("/function/name")
                .and_then(serde_json::Value::as_str)
        })
        .take(256)
        .map(str::to_string)
        .collect()
}

fn is_git_repository(working_dir: &Path) -> bool {
    git2::Repository::discover(working_dir).is_ok()
}

fn send(on_event: &AgentEventEmitter, status: &str) {
    let _ = on_event.send(StreamEvent::Compressing {
        status: status.to_string(),
    });
}
