use super::*;
use crate::services::agent_local::stream_buffer::StreamEventSink;
use crate::services::agent_local::types_ollama::{StreamEvent, StreamOutcome};

struct NoopSink;

impl StreamEventSink for NoopSink {
    fn send_event(&self, _event: StreamEvent) -> Result<(), String> {
        Ok(())
    }
}

#[test]
fn protocol_metadata_is_not_reported_as_partial_output() {
    let mut accumulator = StreamAccumulator::new("gpt-5.6-sol", &[], false, None);

    let outcome = accumulator
        .apply(&NoopSink, &serde_json::json!({"type": "response.created"}))
        .unwrap();

    assert!(outcome.is_none());
    assert!(!accumulator.has_partial_output());
}

#[test]
fn content_and_usage_are_accumulated_until_completion() {
    let mut accumulator = StreamAccumulator::new("gpt-5.6-sol", &[], false, None);
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
fn incomplete_failed_and_error_events_are_rejected() {
    for event in [
        serde_json::json!({"type": "response.incomplete"}),
        serde_json::json!({"type": "response.failed"}),
        serde_json::json!({"type": "error"}),
    ] {
        let mut accumulator = StreamAccumulator::new("gpt-5.6-sol", &[], false, None);
        assert!(accumulator.apply(&NoopSink, &event).is_err());
    }
}

#[test]
fn accumulated_text_is_bounded() {
    let mut accumulator = StreamAccumulator::new("gpt-5.6-sol", &[], false, None);
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
    let mut accumulator = StreamAccumulator::new("gpt-5.6-sol", &[], false, Some(budget));
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
fn completed_tool_call_counts_as_partial_output() {
    let mut accumulator = StreamAccumulator::new("gpt-5.6-sol", &[], false, None);
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
}
