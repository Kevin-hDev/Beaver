use std::path::Path;
use std::time::Instant;

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
    let estimated = super::token_estimate::estimate_textual_request_tokens_for_provider(
        request.provider_id,
        request.runtime_messages,
        request.provider_tools,
    );
    let _provider_usage = request.last_context_tokens;
    let used = estimated;
    let system_head_tokens = super::orchestrator_support::system_head_tokens(
        request.provider_id,
        request.runtime_messages,
        request.provider_tools,
    );
    if !profile.available(request.context_window) {
        return match request.trigger {
            CompressionTrigger::Automatic => Ok(None),
            CompressionTrigger::Explicit => Err(CompressionError::UnavailableUnder64K),
        };
    }
    if !eligible(&profile, request.trigger, request.context_window, used) {
        return Ok(None);
    }
    if !preflight_messages(&session.messages, request.trigger)? {
        return Ok(None);
    }
    let prepared_guard = match super::automatic_guard::prepare(
        &session,
        &profile,
        request.context_window,
        request.trigger,
    )
    .await
    {
        Ok(value) => value,
        Err(CompressionError::AutomaticSuspended) => {
            let _ = request.on_event.send(StreamEvent::Notice {
                message_key: "errors.compressionAutomaticSuspended".into(),
            });
            return Err(CompressionError::AutomaticSuspended);
        }
        Err(error) => return Err(error),
    };
    let Some(prepared_guard) = prepared_guard else {
        return Ok(None);
    };
    let session = prepared_guard.session;
    let attempt = prepared_guard.attempt;
    let started_at = Instant::now();
    let session_id = request.session_id.to_string();
    let request_id = request.request_id.to_string();
    let provider_id = request.provider_id.to_string();
    let trigger = request.trigger;
    let context_window = request.context_window;
    let cancelled = request.cancel.clone();
    let compression_count = session.compression_count;
    let cache_before =
        crate::services::provider_usage::compression_cache_totals(&provider_id).await;
    let on_event = request.on_event;
    super::orchestrator_support::send(on_event, "start");
    crate::services::agent_local::stream_diagnostics::mark_phase(
        request.session_id,
        request.request_id,
        "compression",
        "Compression du contexte démarrée.",
    )
    .await;
    let result =
        super::orchestrator_started::run(request, session, profile.clone(), used, attempt.clone())
            .await;
    if result.is_err() {
        if let Some(attempt) = attempt.as_ref() {
            super::automatic_guard::record_failure(&session_id, attempt).await;
        }
    }
    super::orchestrator_metrics::record(super::orchestrator_metrics::Completion {
        session_id: &session_id,
        request_id: &request_id,
        provider_id: &provider_id,
        profile: &profile,
        trigger,
        context_window,
        before_tokens: used.min(u32::MAX as usize) as u32,
        system_head_tokens,
        previous_compression_count: compression_count,
        cache_before,
        facts: result.as_ref().ok().map(|value| value.facts),
        error: result.as_ref().err().copied(),
        cancelled: result.is_err() && cancelled.is_cancelled(),
        started_at,
    })
    .await;
    super::orchestrator_support::send(on_event, "done");
    if result.is_ok() {
        let _ = on_event.send(StreamEvent::CompressionComplete {});
    }
    result.map(|value| Some(value.report))
}

pub(super) fn preflight_messages(
    messages: &[crate::services::agent_local::types_message::AgentMessage],
    trigger: CompressionTrigger,
) -> Result<bool, CompressionError> {
    if crate::services::agent_local::conversation_history_validation::validate(messages).is_ok() {
        return Ok(true);
    }
    match trigger {
        CompressionTrigger::Automatic => Ok(false),
        CompressionTrigger::Explicit => Err(CompressionError::OpenTurn),
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
            profile.automatic_enabled
                && context_window > 0
                && super::token_estimate::should_compress(
                    used_tokens,
                    context_window,
                    profile.profile.threshold_percent.min(90),
                )
        }
    }
}
