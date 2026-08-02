use super::request_measurement::{RequestMeasurement, RequestMeasurementContext};
use super::{UsageApiFormat, UsageWorkload};
use serde_json::json;

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
    }
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
