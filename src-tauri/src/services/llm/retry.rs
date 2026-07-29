//! Logique de retry pour les appels LLM API.
//!
//! Gère les erreurs transitoires (429, 503, timeout) avec back-off progressif.

use super::stream;
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::{ChatMessage, StreamOutcome};
use crate::services::compress::realtime_budget::RealtimeBudget;
use crate::services::llm::request_purpose::RequestPurpose;
use tokio_util::sync::CancellationToken;

const MAX_RETRIES: usize = 3;
const RETRY_BASE_MS: u64 = 2000;

fn is_retryable_error(error: &str) -> bool {
    matches!(
        error,
        "rate_limit" | "provider_temporarily_unavailable" | "provider_connection_failed"
    )
}

pub async fn retry_stream(
    on_event: &AgentEventEmitter,
    session_id: &str,
    request_id: &str,
    provider_id: &str,
    purpose: RequestPurpose,
    model: &str,
    messages: &[ChatMessage],
    tools: &[serde_json::Value],
    think: bool,
    reasoning_mode: Option<&str>,
    cancel: CancellationToken,
    buffer_content: bool,
    realtime_budget: Option<RealtimeBudget>,
) -> Result<StreamOutcome, String> {
    let mut last_error = String::new();
    for attempt in 0..=MAX_RETRIES {
        if cancel.is_cancelled() {
            return Err("Annulé".to_string());
        }
        if attempt > 0 {
            eprintln!("[llm retry] attempt={attempt}/{MAX_RETRIES}");
            crate::services::agent_local::stream_diagnostics::record_retry(
                session_id,
                request_id,
                "Nouvelle tentative provider.",
            )
            .await;
            crate::services::agent_local::ollama_retry_indicator::send_retry_indicator(
                on_event,
                crate::services::agent_local::ollama_retry_indicator::REASON_PROVIDER,
                attempt as u32,
                MAX_RETRIES as u32,
            );
            let delay = RETRY_BASE_MS * (1 << (attempt - 1));
            wait_for_retry(&cancel, tokio::time::Duration::from_millis(delay)).await?;
        }
        match stream::stream_chat_no_done(
            on_event,
            provider_id,
            purpose,
            model,
            messages,
            tools,
            think,
            reasoning_mode,
            cancel.clone(),
            buffer_content,
            realtime_budget.clone(),
        )
        .await
        {
            Ok(result) => return Ok(result),
            Err(e) if is_retryable_error(&e) && attempt < MAX_RETRIES => {
                last_error = e;
                continue;
            }
            Err(e) => return Err(e),
        }
    }
    Err(last_error)
}

async fn wait_for_retry(
    cancel: &CancellationToken,
    delay: tokio::time::Duration,
) -> Result<(), String> {
    tokio::select! {
        _ = cancel.cancelled() => Err("Annulé".to_string()),
        _ = tokio::time::sleep(delay) => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::{is_retryable_error, wait_for_retry};
    use tokio_util::sync::CancellationToken;

    #[test]
    fn stable_temporary_codes_are_retryable() {
        assert!(is_retryable_error("provider_temporarily_unavailable"));
        assert!(is_retryable_error("provider_connection_failed"));
    }

    #[test]
    fn permanent_request_errors_are_not_retried() {
        assert!(!is_retryable_error("Codex: Invalid request."));
        assert!(!is_retryable_error("SSE: private transport details"));
    }

    #[tokio::test]
    async fn retry_wait_stops_as_soon_as_the_stream_is_cancelled() {
        let cancel = CancellationToken::new();
        cancel.cancel();

        let result = wait_for_retry(&cancel, tokio::time::Duration::from_secs(30)).await;

        assert_eq!(result.unwrap_err(), "Annulé");
    }
}
