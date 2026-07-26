//! Logique de retry pour les appels LLM API.
//!
//! Gère les erreurs transitoires (429, 503, timeout) avec back-off progressif.

use super::stream;
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::{ChatMessage, StreamOutcome};
use crate::services::compress::realtime_budget::RealtimeBudget;
use crate::services::llm::request_purpose::RequestPurpose;
use tokio_util::sync::CancellationToken;

const MAX_RETRIES: usize = 5;
const RETRY_BASE_MS: u64 = 2000;

fn is_retryable_error(error: &str) -> bool {
    let lower = error.to_ascii_lowercase();
    [
        "429",
        "rate limit",
        "503",
        "502",
        "timeout",
        "etimedout",
        "econnreset",
        "transport error",
        "decoding response body",
        "connection closed",
        "sse:",
        "overloaded",
        "temporarily unavailable",
        "provider_temporarily_unavailable",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
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
            eprintln!("[llm retry] attempt={attempt}/{MAX_RETRIES} error={last_error}");
            crate::services::agent_local::stream_diagnostics::record_retry(
                session_id,
                request_id,
                "Nouvelle tentative provider.",
            )
            .await;
            let delay = RETRY_BASE_MS * (1 << (attempt - 1));
            tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
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

#[cfg(test)]
mod tests {
    use super::is_retryable_error;

    #[test]
    fn codex_overload_is_retryable() {
        assert!(is_retryable_error(
            "Codex: Our servers are currently overloaded. Please try again later."
        ));
    }

    #[test]
    fn permanent_request_errors_are_not_retried() {
        assert!(!is_retryable_error("Codex: Invalid request."));
    }

    #[test]
    fn codex_http_503_and_high_demand_are_retryable() {
        assert!(is_retryable_error("provider_temporarily_unavailable"));
    }
}
