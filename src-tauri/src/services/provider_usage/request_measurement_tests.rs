use super::request_journal::ServiceTierServed;
use super::request_measurement::{RequestMeasurement, RequestMeasurementContext};
use super::{UsageApiFormat, UsageWorkload};
use crate::services::llm::fast_mode::FastModeRequest;
use serde_json::json;

#[test]
fn response_failure_preserves_safe_details_and_exact_connection_on_disk() {
    let request_id = uuid::Uuid::new_v4().to_string();
    let mut request = context(None);
    request.connection_id = "xai-oauth";
    request.canonical_provider_id = "xai";
    request.api_format = UsageApiFormat::Responses;
    request.model = "grok-4.6";
    request.request_id = &request_id;
    let mut measurement = RequestMeasurement::start(request).unwrap();
    measurement.observe_response_metadata(&json!({
        "type": "response.failed",
        "response": {"error": {
            "type": "invalid_request_error",
            "code": "unsupported_parameter",
            "param": "include",
            "message": "private content sk-test-do-not-persist-1234567890"
        }}
    }));

    let path = crate::services::paths::data_dir().join("logs/provider-errors.jsonl");
    let bytes = std::fs::read_to_string(path).unwrap_or_default();
    let saved = bytes
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .find(|entry| entry["request_id"] == request_id)
        .expect("the real response failure must leave a correlated diagnostic");
    assert_eq!(saved["provider"], "xai-oauth");
    assert_eq!(saved["model"], "grok-4.6");
    assert_eq!(saved["transport"], "stream");
    assert_eq!(saved["details"]["error_code"], "unsupported_parameter");
    assert_eq!(saved["details"]["error_param"], "include");
    assert!(
        saved.get("status").is_none(),
        "do not invent an HTTP failure status"
    );
    assert!(!saved.to_string().contains("private content"));
    assert!(!saved.to_string().contains("sk-test"));
}

#[test]
fn response_failure_accepts_top_level_error_without_recording_regular_events() {
    let request_id = uuid::Uuid::new_v4().to_string();
    let mut request = context(None);
    request.request_id = &request_id;
    let mut measurement = RequestMeasurement::start(request).unwrap();
    let path = crate::services::paths::data_dir().join("logs/provider-errors.jsonl");
    measurement.observe_response_metadata(&json!({
        "type": "response.output_text.delta", "delta": "private content"
    }));
    let before = std::fs::read_to_string(&path).unwrap_or_default();
    assert!(!before.contains(&request_id));

    measurement.observe_response_metadata(&json!({
        "type": "error", "code": "resource-exhausted",
        "param": "sk-live-1234567890abcdefghijklmnop",
        "message": "private content"
    }));
    let bytes = std::fs::read_to_string(path).unwrap();
    let saved = bytes
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .find(|entry| entry["request_id"] == request_id)
        .unwrap();
    assert_eq!(saved["details"]["error_code"], "resource-exhausted");
    assert!(!saved.to_string().contains("sk-live"));
    assert!(!saved.to_string().contains("private content"));
}

#[tokio::test]
async fn live_openai_responses_measurement_is_persisted() {
    let id = uuid::Uuid::new_v4().to_string();
    let mut request = context(Some(&id));
    request.request_id = &id;
    request.api_format = UsageApiFormat::Responses;
    request.model = "gpt-6-astra";
    let measurement = RequestMeasurement::start(request).expect("Responses must be measured");
    measurement
        .finish(
            super::RequestMetricStatus::Completed,
            Some(&super::RequestUsage {
                input_tokens: Some(100),
                output_tokens: Some(10),
                ..Default::default()
            }),
            true,
        )
        .await;
    let snapshot = super::request_journal::snapshot("openai").await;
    let saved = snapshot
        .recent
        .iter()
        .find(|entry| entry.request_id == id)
        .unwrap();
    assert_eq!(saved.api_format, UsageApiFormat::Responses);
    assert_eq!(saved.usage.as_ref().unwrap().input_tokens, Some(100));
    let mut invalid = context(None);
    invalid.api_format = UsageApiFormat::AnthropicMessages;
    assert!(RequestMeasurement::start(invalid).is_none());
    // Historical Chat entries must remain readable after the route migration.
    assert!(RequestMeasurement::start(context(None)).is_some());
}

fn context<'a>(session_id: Option<&'a str>) -> RequestMeasurementContext<'a> {
    RequestMeasurementContext {
        connection_id: "openai",
        canonical_provider_id: "openai",
        api_format: UsageApiFormat::ChatCompletions,
        model: "gpt-5.6-sol",
        session_id,
        request_id: "request-1",
        turn: Some(1),
        attempt: 1,
        workload: UsageWorkload::Primary,
        fast_mode: FastModeRequest::Fast,
    }
}

#[test]
fn measurement_records_the_captured_request_and_real_served_tier() {
    let mut measurement = RequestMeasurement::start(context(Some("session-1"))).unwrap();

    measurement.observe_response_metadata(&json!({"service_tier": "priority"}));

    assert_eq!(
        measurement.fast_observation(),
        (true, ServiceTierServed::Fast)
    );
}

#[test]
fn unknown_metadata_never_erases_a_known_served_tier() {
    let mut measurement = RequestMeasurement::start(context(Some("session-1"))).unwrap();
    measurement.observe_response_metadata(&json!({
        "response": {"service_tier": "default"}
    }));

    measurement.observe_response_metadata(&json!({"service_tier": "ultrafast"}));

    assert_eq!(
        measurement.fast_observation(),
        (true, ServiceTierServed::Default)
    );
}

#[test]
fn monotonic_timing_keeps_each_available_phase_separate() {
    let mut measurement = RequestMeasurement::start(context(Some("session-1"))).unwrap();

    measurement.mark_headers();
    measurement.mark_first_event();
    measurement.mark_first_useful();

    let timing = measurement.timing();
    assert!(timing.headers_ms.is_some());
    assert!(timing.first_event_ms.is_some());
    assert!(timing.first_useful_ms.is_some());
}

#[test]
fn transport_fallback_never_overwrites_an_observed_milestone() {
    let mut measurement = RequestMeasurement::start(context(Some("session-1"))).unwrap();
    measurement.mark_headers();
    measurement.mark_first_event();
    measurement.mark_first_useful();
    let first = measurement.timing().clone();

    std::thread::sleep(std::time::Duration::from_millis(2));
    measurement.mark_headers();
    measurement.mark_first_event();
    measurement.mark_first_useful();

    assert_eq!(measurement.timing(), &first);
}

#[test]
fn invalid_local_identity_disables_measurement_without_persisting_it() {
    assert!(RequestMeasurement::start(context(Some("../session"))).is_none());
}

#[test]
fn openrouter_keeps_only_the_selected_safe_endpoint() {
    let mut openrouter = context(Some("session-1"));
    openrouter.connection_id = "openrouter";
    openrouter.canonical_provider_id = "openrouter";
    let mut measurement = RequestMeasurement::start(openrouter).unwrap();

    measurement.observe_response_metadata(&json!({
        "openrouter_metadata": {
            "endpoints": { "available": [
                { "provider": "First", "model": "model-a", "selected": false },
                { "provider": "Google Vertex", "model": "google/gemini-3.5-pro", "selected": true }
            ] }
        }
    }));

    assert_eq!(
        measurement.routed_endpoint(),
        (Some("Google Vertex"), Some("google/gemini-3.5-pro")),
    );
}

#[test]
fn unsafe_openrouter_metadata_is_ignored() {
    let mut openrouter = context(Some("session-1"));
    openrouter.connection_id = "openrouter";
    openrouter.canonical_provider_id = "openrouter";
    let mut measurement = RequestMeasurement::start(openrouter).unwrap();

    measurement.observe_response_metadata(&json!({
        "openrouter_metadata": { "endpoints": { "available": [
            { "provider": "bad\nvalue", "model": "model-a", "selected": true }
        ] } }
    }));

    assert_eq!(measurement.routed_endpoint(), (None, None));
}

#[test]
fn provider_request_metadata_accepts_only_bounded_safe_labels() {
    let mut measurement = RequestMeasurement::start(context(Some("session-1"))).unwrap();

    measurement.observe_provider_request_id("req_123.abc");
    measurement.observe_finish_reason("tool_use");
    assert_eq!(
        measurement.provider_metadata(),
        (Some("req_123.abc"), Some("tool_use"))
    );

    measurement.observe_provider_request_id("bad/value");
    measurement.observe_finish_reason(&"x".repeat(129));
    assert_eq!(
        measurement.provider_metadata(),
        (Some("req_123.abc"), Some("tool_use"))
    );
}
