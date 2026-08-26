use super::*;
use crate::services::llm::fast_mode::FastModeRequest;

fn request(fast_mode: FastModeRequest) -> CodexRequest {
    crate::services::codex_client::request::build_codex_request(
        "gpt-5.6-sol",
        &[],
        &[],
        None,
        Some("session-test"),
        fast_mode,
    )
}

#[test]
fn websocket_payload_uses_the_current_response_create_envelope() {
    let payload = build_payload(&request(FastModeRequest::Fast)).unwrap();
    let value: serde_json::Value = serde_json::from_str(&payload).unwrap();

    assert_eq!(value["type"], "response.create");
    assert_eq!(value["model"], "gpt-5.6-sol");
    assert_eq!(value["service_tier"], "priority");
    assert_eq!(value["stream"], true);
    assert_eq!(value["tools"], serde_json::json!([]));
    assert_eq!(value["tool_choice"], "auto");
    assert_eq!(value["parallel_tool_calls"], false);
    assert!(value["prompt_cache_key"]
        .as_str()
        .is_some_and(|key| key.starts_with("bv1_")));
}

#[test]
fn websocket_standard_payload_omits_service_tier() {
    let payload = build_payload(&request(FastModeRequest::Standard)).unwrap();
    let value: serde_json::Value = serde_json::from_str(&payload).unwrap();

    assert_eq!(value["type"], "response.create");
    assert!(value.get("service_tier").is_none());
}

#[test]
fn websocket_cooldown_is_bounded_and_expires() {
    let deadline = cooldown_deadline(1_000);

    assert!(cooldown_active(1_001, deadline));
    assert!(!cooldown_active(deadline, deadline));
    assert_eq!(deadline, 1_000 + WEBSOCKET_COOLDOWN_MS);
}

#[test]
fn fallback_knows_whether_partial_output_must_be_cleared() {
    assert!(WebSocketFailure::Unavailable { partial: true }.has_partial_output());
    assert!(!WebSocketFailure::Unavailable { partial: false }.has_partial_output());
    assert!(!WebSocketFailure::Cancelled.has_partial_output());
    assert!(!WebSocketFailure::ProviderRejected {
        code: crate::services::llm::provider_error::ProviderErrorCode::ServiceTierUnavailable,
    }
    .has_partial_output());
}

#[test]
fn accumulator_keeps_only_closed_permanent_provider_codes() {
    assert_eq!(
        accumulator_failure("service_tier_unavailable", false),
        WebSocketFailure::ProviderRejected {
            code: crate::services::llm::provider_error::ProviderErrorCode::ServiceTierUnavailable,
        }
    );
    assert_eq!(
        accumulator_failure("private upstream detail", false),
        WebSocketFailure::Unavailable { partial: false }
    );
}

#[tokio::test]
async fn invalid_routing_configuration_is_a_permanent_rejection() {
    let emitter = crate::services::agent_local::stream_events::AgentEventEmitter::test(
        "session-invalid-routing".into(),
    );
    let mut measurement =
        crate::services::codex_client::stream_measurement::StreamMeasurement::new(None);

    let error = stream_chat(
        &emitter,
        "session-invalid-routing",
        "../invalid-model",
        &[],
        &[],
        None,
        FastModeRequest::Fast,
        tokio_util::sync::CancellationToken::new(),
        false,
        None,
        &mut measurement,
    )
    .await
    .unwrap_err();

    assert_eq!(
        error,
        WebSocketFailure::ProviderRejected {
            code: crate::services::llm::provider_error::ProviderErrorCode::ProviderConfigurationInvalid,
        }
    );
}

#[tokio::test]
async fn invalid_routing_configuration_never_disables_websocket_or_falls_back() {
    mark_available();
    let emitter = crate::services::agent_local::stream_events::AgentEventEmitter::test(
        "session-invalid-routing-orchestration".into(),
    );

    let error = crate::services::codex_client::stream::stream_chat_with_budget(
        &emitter,
        "session-invalid-routing-orchestration",
        "request-invalid-routing-orchestration",
        "../invalid-model",
        &[],
        &[],
        None,
        FastModeRequest::Fast,
        tokio_util::sync::CancellationToken::new(),
        false,
        None,
        None,
        None,
        None,
    )
    .await
    .unwrap_err();

    assert_eq!(error, "provider_configuration_invalid");
    assert!(should_attempt());
}
