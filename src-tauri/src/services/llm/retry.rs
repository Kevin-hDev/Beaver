#![expect(
    clippy::too_many_arguments,
    reason = "orchestration boundary keeps related runtime context explicit"
)]
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
    cancel: CancellationToken,
    buffer_content: bool,
    realtime_budget: Option<RealtimeBudget>,
    continuation_target: Option<
        &crate::services::reasoning_continuity::contract::ContinuationTarget,
    >,
) -> Result<StreamOutcome, String> {
    let policy = retry_policy(provider_id);
    let request_target =
        super::reasoning_wire::replay::target_for_request(messages, continuation_target);
    let mut attempt = 0_usize;
    loop {
        if cancel.is_cancelled() {
            return Err("Annulé".to_string());
        }
        if attempt > 0 {
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
        let outbound_attempt = take_attempt(next_attempt);
        match stream::stream_chat_no_done(
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
            cancel.clone(),
            buffer_content,
            realtime_budget.clone(),
            request_target
                .as_ref()
                .and_then(
                    crate::services::reasoning_continuity::contract::ContinuationTarget::replay,
                )
                .map(super::reasoning_wire::ReasoningCaptureContext::from_target)
                .map(super::reasoning_wire::ReasoningCapture::new)
                .transpose()
                .map_err(|_| "provider_configuration_invalid".to_string())?,
            request_target.as_ref(),
        )
        .await
        {
            Ok(result) => return Ok(result),
            Err(e)
                if is_retryable_error(&e)
                    && automatic_retry_allowed(tools)
                    && attempt < policy.max_retries =>
            {
                attempt += 1;
                continue;
            }
            Err(e) => return Err(e),
        }
    }
}

fn automatic_retry_allowed(_tools: &[serde_json::Value]) -> bool {
    // Aucun transport provider ne transmet encore de clé d'idempotence : une
    // erreur réseau peut cacher une facturation ou l'exécution d'un outil.
    false
}

fn take_attempt(next_attempt: &mut u32) -> u32 {
    let current = *next_attempt;
    *next_attempt = next_attempt.saturating_add(1);
    current
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
    use super::{
        automatic_retry_allowed, is_retryable_error, retry_delay, retry_policy, take_attempt,
        wait_for_retry, CODEX_POLICY,
    };
    use tokio_util::sync::CancellationToken;

    #[test]
    fn stable_temporary_codes_are_retryable() {
        assert!(is_retryable_error("provider_temporarily_unavailable"));
        assert!(is_retryable_error("provider_connection_failed"));
    }

    #[test]
    fn permanent_request_errors_are_not_retried() {
        assert!(!is_retryable_error("service_tier_unavailable"));
        assert!(!is_retryable_error("Codex: Invalid request."));
        assert!(!is_retryable_error("SSE: private transport details"));
    }

    #[test]
    fn provider_requests_never_retry_without_an_idempotency_key() {
        assert!(!automatic_retry_allowed(&[]));
        assert!(!automatic_retry_allowed(&[
            serde_json::json!({"type": "function"})
        ]));
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

    #[test]
    fn attempt_sequence_survives_a_second_retry_phase() {
        let mut next = 1;
        assert_eq!(take_attempt(&mut next), 1);
        assert_eq!(take_attempt(&mut next), 2);
        assert_eq!(next, 3);
    }
}

#[cfg(test)]
#[path = "retry_fast_mode_tests.rs"]
mod fast_mode_tests;
