use super::stream_events::AgentEventEmitter;
use super::types_ollama::StreamEvent;
use reqwest::StatusCode;
use std::time::Duration;

pub const REASON_FEATURE_DROPPED: &str = "agentLocal.retry.featureDropped";
pub const REASON_PARSER_CRASH: &str = "agentLocal.retry.parserCrash";
pub const REASON_THINKING_ONLY: &str = "agentLocal.retry.thinkingOnly";
pub const REASON_SERVER: &str = "agentLocal.retry.server";
pub const REASON_PROVIDER: &str = "agentLocal.retry.provider";

pub fn retry_indicator(reason_key: &str, attempt: u32, max_attempts: u32) -> StreamEvent {
    StreamEvent::RetryIndicator {
        reason_key: reason_key.to_string(),
        attempt,
        max_attempts,
    }
}

pub fn send_retry_indicator(
    on_event: &AgentEventEmitter,
    reason_key: &str,
    attempt: u32,
    max_attempts: u32,
) {
    let _ = on_event.send(retry_indicator(reason_key, attempt, max_attempts));
}

pub fn max_server_retries() -> u32 {
    crate::services::llm::route_profile::error_policy("ollama")
        .expect("the built-in Ollama route profile must exist")
        .max_server_retries()
}

pub fn should_retry_server_status(status: StatusCode, retries: u32) -> bool {
    crate::services::llm::route_profile::error_policy("ollama")
        .is_some_and(|policy| policy.allows_server_retry(status.as_u16(), retries))
}

pub fn server_retry_delay(attempt: u32) -> Duration {
    Duration::from_millis(350 * u64::from(attempt.clamp(1, 6)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn retries_only_temporary_server_statuses() {
        assert!(should_retry_server_status(
            StatusCode::INTERNAL_SERVER_ERROR,
            0,
        ));
        assert!(should_retry_server_status(StatusCode::BAD_GATEWAY, 0));
        assert!(should_retry_server_status(StatusCode::SERVICE_UNAVAILABLE, 0));
        assert!(should_retry_server_status(StatusCode::GATEWAY_TIMEOUT, 0));
        assert!(!should_retry_server_status(StatusCode::BAD_REQUEST, 0));
        assert!(!should_retry_server_status(StatusCode::NOT_FOUND, 0));
        assert!(!should_retry_server_status(
            StatusCode::SERVICE_UNAVAILABLE,
            max_server_retries(),
        ));
    }

    #[test]
    fn serializes_retry_indicator_in_camel_case() {
        let event = retry_indicator(REASON_SERVER, 2, max_server_retries());
        assert_eq!(
            serde_json::to_value(event).unwrap(),
            json!({
                "event": "retryIndicator",
                "data": {
                    "reasonKey": "agentLocal.retry.server",
                    "attempt": 2,
                    "maxAttempts": 10
                }
            })
        );
    }
}
