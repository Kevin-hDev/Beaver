use super::*;
use crate::services::agent_local::stream_buffer::StreamEventSink;
use crate::services::agent_local::types_ollama::{StreamEvent, StreamOutcome};
use crate::services::llm::fast_mode::FastModeRequest;
use crate::services::provider_usage::{
    RequestMeasurement, RequestMeasurementContext, UsageApiFormat, UsageWorkload,
};

struct NoopSink;

impl StreamEventSink for NoopSink {
    fn send_event(&self, _event: StreamEvent) -> Result<(), String> {
        Ok(())
    }
}

#[test]
fn responses_consumer_observes_the_tier_before_completion() {
    let mut request_measurement = RequestMeasurement::start(RequestMeasurementContext {
        connection_id: "codex-oauth",
        canonical_provider_id: "openai",
        api_format: UsageApiFormat::Responses,
        model: "gpt-5.6-sol",
        session_id: Some("session-1"),
        request_id: "request-1",
        turn: Some(1),
        attempt: 1,
        workload: UsageWorkload::Primary,
        fast_mode: FastModeRequest::Fast,
    })
    .unwrap();
    let mut accumulator = StreamAccumulator::new("openai", "gpt-5.6-sol", &[], false, None);
    let mut measurement = crate::services::codex_client::stream_measurement::StreamMeasurement::new(
        Some(&mut request_measurement),
    );

    measurement
        .apply(
            &mut accumulator,
            &NoopSink,
            &serde_json::json!({
                "type": "response.completed",
                "response": {"service_tier": "priority"}
            }),
        )
        .unwrap();

    assert_eq!(
        request_measurement.fast_observation().1,
        crate::services::provider_usage::ServiceTierServed::Fast
    );
}

#[test]
fn protocol_metadata_is_not_reported_as_partial_output() {
    let mut accumulator = StreamAccumulator::new("openai", "gpt-5.6-sol", &[], false, None);

    let outcome = accumulator
        .apply(&NoopSink, &serde_json::json!({"type": "response.created"}))
        .unwrap();

    assert!(outcome.is_none());
    assert!(!accumulator.has_partial_output());
}

#[test]
fn content_and_usage_are_accumulated_until_completion() {
    let mut accumulator = StreamAccumulator::new("openai", "gpt-5.6-sol", &[], false, None);
    accumulator
        .apply(
            &NoopSink,
            &serde_json::json!({"type": "response.output_text.delta", "delta": "bonjour"}),
        )
        .unwrap();

    let outcome = accumulator
        .apply(
            &NoopSink,
            &serde_json::json!({
                "type": "response.completed",
                "response": {"usage": {"input_tokens": 12, "output_tokens": 3}}
            }),
        )
        .unwrap()
        .unwrap();
    let StreamOutcome::Completed(result) = outcome else {
        panic!("completion expected");
    };

    assert_eq!(result.content, "bonjour");
    assert_eq!(result.prompt_tokens, Some(12));
    assert_eq!(result.eval_count, Some(3));
}

#[test]
fn completed_responses_stream_persists_native_items_without_tool_extra_content() {
    use crate::services::llm::reasoning_wire::{ReasoningCapture, ReasoningCaptureContext};
    use crate::services::reasoning_continuity::contract::{
        CredentialScope, ReasoningModeId, RouteId,
    };
    use crate::services::reasoning_continuity::envelope::ContinuationState;

    let capture = ReasoningCapture::new(ReasoningCaptureContext {
        route_id: RouteId::OpenAi,
        model_id: "gpt-5.6-luna".into(),
        credential_scope: CredentialScope::authenticated("fixture-scope").unwrap(),
        reasoning_mode: ReasoningModeId::Medium,
    })
    .unwrap();
    let tools = [serde_json::json!({
        "type": "function",
        "function": {
            "name": "fixture.write_note",
            "description": "fixture",
            "parameters": {"type": "object"}
        }
    })];
    let mut accumulator = StreamAccumulator::new_with_capture(
        "openai",
        "gpt-5.6-luna",
        &tools,
        false,
        None,
        Some(capture),
        true,
    );
    accumulator
        .apply(
            &NoopSink,
            &serde_json::json!({
                "type": "response.output_item.done",
                "item": {"type": "reasoning", "encrypted_content": "opaque"}
            }),
        )
        .unwrap();
    accumulator
        .apply(
            &NoopSink,
            &serde_json::json!({
                "type": "response.output_item.added",
                "item": {
                    "type": "function_call",
                    "call_id": "call_1",
                    "name": "fixture_write_note"
                }
            }),
        )
        .unwrap();
    accumulator
        .apply(
            &NoopSink,
            &serde_json::json!({
                "type": "response.output_item.done",
                "item": {
                    "type": "function_call",
                    "call_id": "call_1",
                    "name": "fixture_write_note",
                    "arguments": "{}"
                }
            }),
        )
        .unwrap();
    let outcome = accumulator
        .apply(
            &NoopSink,
            &serde_json::json!({
                "type": "response.output_item.done",
                "item": {"type": "message", "content": []}
            }),
        )
        .unwrap();
    assert!(outcome.is_none());
    let outcome = accumulator
        .apply(
            &NoopSink,
            &serde_json::json!({"type": "response.completed"}),
        )
        .unwrap()
        .unwrap();
    let StreamOutcome::Completed(result) = outcome else {
        panic!("completion expected");
    };
    let Some(continuation) = result.continuation else {
        panic!("native continuation expected");
    };
    assert_eq!(continuation.tool_links.len(), 1);
    assert_eq!(continuation.tool_links[0].provider_call_id, "call_1");
    assert_eq!(continuation.tool_links[0].tool_name, "fixture.write_note");
    let ContinuationState::ResponsesLocal { items } = continuation.continuation else {
        panic!("Responses continuation expected");
    };
    assert_eq!(items.len(), 3);
    assert!(result.tool_call_extra_content.is_empty());
}

#[test]
fn legacy_codex_replay_is_used_only_without_an_active_native_capture() {
    use crate::services::llm::reasoning_wire::{ReasoningCapture, ReasoningCaptureContext};
    use crate::services::reasoning_continuity::contract::{
        CredentialScope, ReasoningModeId, RouteId,
    };

    let start_event = serde_json::json!({
        "type": "response.output_item.added",
        "item": {"type": "function_call", "call_id": "call_1", "name": "lookup"}
    });
    let tool_event = serde_json::json!({
        "type": "response.output_item.done",
        "item": {
            "type": "function_call",
            "call_id": "call_1",
            "name": "lookup",
            "arguments": "{}"
        }
    });
    let mut legacy =
        StreamAccumulator::new_with_capture("openai", "gpt-5.6-luna", &[], false, None, None, true);
    legacy.apply(&NoopSink, &start_event).unwrap();
    legacy.apply(&NoopSink, &tool_event).unwrap();
    let legacy = legacy
        .apply(
            &NoopSink,
            &serde_json::json!({"type": "response.completed"}),
        )
        .unwrap()
        .unwrap()
        .into_result();
    assert!(legacy.tool_call_extra_content[0].is_some());

    let capture = ReasoningCapture::new(ReasoningCaptureContext {
        route_id: RouteId::CodexOauth,
        model_id: "gpt-5.6-luna".into(),
        credential_scope: CredentialScope::authenticated("fixture-scope").unwrap(),
        reasoning_mode: ReasoningModeId::Medium,
    })
    .unwrap();
    let mut native = StreamAccumulator::new_with_capture(
        "openai",
        "gpt-5.6-luna",
        &[],
        false,
        None,
        Some(capture),
        true,
    );
    native.apply(&NoopSink, &start_event).unwrap();
    native.apply(&NoopSink, &tool_event).unwrap();
    let native = native
        .apply(
            &NoopSink,
            &serde_json::json!({"type": "response.completed"}),
        )
        .unwrap()
        .unwrap()
        .into_result();
    assert!(native.tool_call_extra_content.is_empty());
    assert!(native.continuation.is_some());
}

#[test]
fn incomplete_failed_and_error_events_are_rejected() {
    for event in [
        serde_json::json!({"type": "response.incomplete"}),
        serde_json::json!({"type": "response.failed"}),
        serde_json::json!({"type": "error"}),
    ] {
        let mut accumulator = StreamAccumulator::new("openai", "gpt-5.6-sol", &[], false, None);
        assert!(accumulator.apply(&NoopSink, &event).is_err());
    }
}

#[test]
fn accumulated_text_is_bounded() {
    let mut accumulator = StreamAccumulator::new("openai", "gpt-5.6-sol", &[], false, None);
    accumulator.text_bytes = MAX_STREAM_TEXT_BYTES - 1;

    assert!(accumulator.record_text_size("x").is_ok());
    assert_eq!(
        accumulator.record_text_size("x").unwrap_err(),
        "provider_payload_too_large"
    );
}

#[test]
fn realtime_budget_interrupts_content_for_compression() {
    let budget = RealtimeBudget::new(true, 100, 1, 0).unwrap();
    let mut accumulator = StreamAccumulator::new("openai", "gpt-5.6-sol", &[], false, Some(budget));
    let content = "x".repeat(256);

    let outcome = accumulator
        .apply(
            &NoopSink,
            &serde_json::json!({"type": "response.output_text.delta", "delta": content}),
        )
        .unwrap()
        .unwrap();

    assert!(matches!(
        outcome,
        StreamOutcome::InterruptedForCompression(_)
    ));
}

#[test]
fn started_tool_call_is_useful_but_only_completion_is_partial_output() {
    let mut accumulator = StreamAccumulator::new("openai", "gpt-5.6-sol", &[], false, None);
    accumulator
        .apply(
            &NoopSink,
            &serde_json::json!({
                "type": "response.output_item.added",
                "item": {"type": "function_call", "call_id": "call_1", "name": "bash"}
            }),
        )
        .unwrap();
    assert!(!accumulator.has_partial_output());
    assert!(accumulator.has_useful_output());

    accumulator
        .apply(
            &NoopSink,
            &serde_json::json!({
                "type": "response.output_item.done",
                "item": {
                    "type": "function_call",
                    "call_id": "call_1",
                    "name": "bash",
                    "arguments": "{\"cmd\":\"pwd\"}"
                }
            }),
        )
        .unwrap();

    assert!(accumulator.has_partial_output());
    assert!(accumulator.has_useful_output());
}
