#![expect(
    clippy::too_many_arguments,
    reason = "orchestration boundary keeps related runtime context explicit"
)]
//! Frontière historique d'appel provider.
//!
//! Les reprises automatiques restent fermées tant qu'un transport ne transmet
//! pas une clé d'idempotence : une réponse perdue peut déjà avoir été facturée.

use super::stream;
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::{ChatMessage, StreamOutcome};
use crate::services::compress::realtime_budget::RealtimeBudget;
use crate::services::llm::request_purpose::RequestPurpose;
use tokio_util::sync::CancellationToken;

pub async fn retry_stream(
    on_event: &AgentEventEmitter,
    session_id: &str,
    request_id: &str,
    turn: u32,
    next_attempt: &mut u32,
    provider_id: &str,
    fast_mode: super::fast_mode::FastModeRequest,
    purpose: RequestPurpose,
    model: &str,
    messages: &[ChatMessage],
    tools: &[serde_json::Value],
    think: bool,
    reasoning_mode: Option<&str>,
    tool_result_previews: &crate::services::agent_local::tool_artifact_preview::ToolResultPreviewBatch,
    cancel: CancellationToken,
    buffer_content: bool,
    realtime_budget: Option<RealtimeBudget>,
    continuation_target: Option<
        &crate::services::reasoning_continuity::contract::ContinuationTarget,
    >,
) -> Result<StreamOutcome, String> {
    if cancel.is_cancelled() {
        return Err("Annulé".to_string());
    }
    let request_target =
        super::reasoning_wire::replay::target_for_request(messages, continuation_target);
    let outbound_attempt = *next_attempt;
    *next_attempt = next_attempt.saturating_add(1);
    stream::stream_chat_no_done(
        on_event,
        session_id,
        request_id,
        turn,
        outbound_attempt,
        provider_id,
        fast_mode,
        purpose,
        model,
        messages,
        tools,
        think,
        reasoning_mode,
        tool_result_previews,
        cancel,
        buffer_content,
        realtime_budget,
        request_target
            .as_ref()
            .and_then(crate::services::reasoning_continuity::contract::ContinuationTarget::replay)
            .map(super::reasoning_wire::ReasoningCaptureContext::from_target)
            .map(super::reasoning_wire::ReasoningCapture::new)
            .transpose()
            .map_err(|_| "provider_configuration_invalid".to_string())?,
        request_target.as_ref(),
    )
    .await
}

#[cfg(test)]
mod tests {
    #[test]
    fn source_documents_the_idempotency_gate() {
        let source = include_str!("retry.rs");
        let runtime = source.split("#[cfg(test)]").next().expect("runtime source");
        assert!(runtime.contains("clé d'idempotence"));
        assert!(!runtime.contains("tokio::time::sleep"));
    }
}

#[cfg(test)]
#[path = "retry_fast_mode_tests.rs"]
mod fast_mode_tests;
