//! Logique de retry pour les appels LLM API.
//!
//! Gère les erreurs transitoires (429, 503, timeout) avec back-off progressif.

use super::stream;
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::{ChatMessage, StreamOutcome};
use crate::services::compress::realtime_budget::RealtimeBudget;
use crate::services::llm::request_purpose::RequestPurpose;
use rand::Rng;
use tokio_util::sync::CancellationToken;

const DEFAULT_POLICY: RetryPolicy = RetryPolicy {
    max_retries: 3,
    base_delay_ms: 2_000,
};
const CODEX_POLICY: RetryPolicy = RetryPolicy {
    max_retries: 5,
    base_delay_ms: 200,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RetryPolicy {
    max_retries: usize,
    base_delay_ms: u64,
}

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
    let policy = retry_policy(provider_id);
    let mut last_error = String::new();
    for attempt in 0..=policy.max_retries {
        if cancel.is_cancelled() {
            return Err("Annulé".to_string());
        }
        if attempt > 0 {
            eprintln!("[llm retry] attempt={attempt}/{}", policy.max_retries);
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
                policy.max_retries as u32,
            );
            wait_for_retry(&cancel, retry_delay(policy, attempt)).await?;
        }
        match stream::stream_chat_no_done(
            on_event,
            session_id,
            request_id,
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
            Err(e) if is_retryable_error(&e) && attempt < policy.max_retries => {
                last_error = e;
                continue;
            }
            Err(e) => return Err(e),
        }
    }
    Err(last_error)
}

fn retry_policy(provider_id: &str) -> RetryPolicy {
    if provider_id == crate::services::codex_client::PROVIDER_ID {
        CODEX_POLICY
    } else {
        DEFAULT_POLICY
    }
}

fn retry_delay(policy: RetryPolicy, attempt: usize) -> tokio::time::Duration {
    let factor = 1_u64 << attempt.saturating_sub(1).min(10);
    let base = policy.base_delay_ms.saturating_mul(factor);
    let mut rng = rand::rngs::OsRng;
    let jitter_percent = rng.gen_range(90_u64..=110);
    tokio::time::Duration::from_millis(base.saturating_mul(jitter_percent) / 100)
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
    use super::{is_retryable_error, retry_delay, retry_policy, wait_for_retry, CODEX_POLICY};
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

    #[test]
    fn codex_uses_the_current_short_five_retry_policy() {
        assert_eq!(
            retry_policy(crate::services::codex_client::PROVIDER_ID),
            CODEX_POLICY
        );
        assert_eq!(CODEX_POLICY.max_retries, 5);
        let first = retry_delay(CODEX_POLICY, 1).as_millis();
        let fifth = retry_delay(CODEX_POLICY, 5).as_millis();
        assert!((180..=220).contains(&first));
        assert!((2_880..=3_520).contains(&fifth));
    }

    #[tokio::test]
    async fn retry_wait_stops_as_soon_as_the_stream_is_cancelled() {
        let cancel = CancellationToken::new();
        cancel.cancel();

        let result = wait_for_retry(&cancel, tokio::time::Duration::from_secs(30)).await;

        assert_eq!(result.unwrap_err(), "Annulé");
    }
}
