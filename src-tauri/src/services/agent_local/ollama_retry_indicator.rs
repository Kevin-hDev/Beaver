use super::stream_events::AgentEventEmitter;
use super::types_ollama::StreamEvent;
use reqwest::StatusCode;
use std::time::Duration;

pub const MAX_SERVER_RETRIES: u32 = 10;

pub const REASON_FEATURE_DROPPED: &str = "agentLocal.retry.featureDropped";
pub const REASON_PARSER_CRASH: &str = "agentLocal.retry.parserCrash";
pub const REASON_THINKING_ONLY: &str = "agentLocal.retry.thinkingOnly";
pub const REASON_SERVER: &str = "agentLocal.retry.server";

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

pub fn should_retry_server_status(status: StatusCode) -> bool {
    matches!(
        status,
        StatusCode::INTERNAL_SERVER_ERROR
            | StatusCode::BAD_GATEWAY
            | StatusCode::SERVICE_UNAVAILABLE
            | StatusCode::GATEWAY_TIMEOUT
    )
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
            StatusCode::INTERNAL_SERVER_ERROR
        ));
        assert!(should_retry_server_status(StatusCode::BAD_GATEWAY));
        assert!(should_retry_server_status(StatusCode::SERVICE_UNAVAILABLE));
        assert!(should_retry_server_status(StatusCode::GATEWAY_TIMEOUT));
        assert!(!should_retry_server_status(StatusCode::BAD_REQUEST));
        assert!(!should_retry_server_status(StatusCode::NOT_FOUND));
    }

    #[test]
    fn serializes_retry_indicator_in_camel_case() {
        let event = retry_indicator(REASON_SERVER, 2, MAX_SERVER_RETRIES);
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
