//! Frontière d'un appel provider interactif unique.
//!
//! Les reprises automatiques restent fermées tant qu'un transport ne transmet
//! pas une clé d'idempotence : une réponse perdue peut déjà avoir été facturée.

use super::stream;
use crate::services::agent_local::types_ollama::StreamOutcome;

pub async fn stream_once(
    next_attempt: &mut u32,
    context: stream::InteractiveStreamRequest<'_>,
) -> Result<StreamOutcome, String> {
    if context.cancel.is_cancelled() {
        return Err("Annulé".to_string());
    }
    let request_target = super::reasoning_wire::replay::target_for_request(
        context.request.messages,
        context.request.continuation_target,
    );
    let outbound_attempt = *next_attempt;
    *next_attempt = next_attempt.saturating_add(1);
    stream::stream_chat_no_done(
        context,
        outbound_attempt,
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
        let source = include_str!("stream_once.rs");
        let runtime = source.split("#[cfg(test)]").next().expect("runtime source");
        assert!(runtime.contains("clé d'idempotence"));
        assert!(!runtime.contains("tokio::time::sleep"));
    }
}

#[cfg(test)]
#[path = "stream_once_fast_mode_tests.rs"]
mod fast_mode_tests;
