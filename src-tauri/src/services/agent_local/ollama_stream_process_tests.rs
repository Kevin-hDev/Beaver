use super::ollama_stream_process::{done_generation_duration, process_chunk};
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::StreamResult;
use crate::services::stream_utils::ThinkTagFilter;

#[test]
fn reads_bounded_native_ollama_generation_duration() {
    let chunk = serde_json::json!({ "eval_duration": 2_500_000_000_u64 });

    assert_eq!(done_generation_duration(&chunk), Some(2_500_000_000));
}

#[test]
fn rejects_invalid_native_ollama_generation_duration() {
    assert_eq!(done_generation_duration(&serde_json::json!({ "eval_duration": 0 })), None);
}

#[test]
fn disabled_capture_does_not_create_a_continuation_envelope() {
    let mut result = StreamResult::default();
    let mut token_count = 0;
    let mut filter = ThinkTagFilter::new();
    let emitter = AgentEventEmitter::test("session".into());

    process_chunk(
        r#"{"message":{"thinking":"raisonnement affichable"},"done":false}"#,
        &emitter,
        &mut token_count,
        &mut result,
        None,
        &mut filter,
        true,
        None,
    )
    .unwrap();
    process_chunk(
        r#"{"done":true,"done_reason":"stop"}"#,
        &emitter,
        &mut token_count,
        &mut result,
        None,
        &mut filter,
        true,
        None,
    )
    .unwrap();

    assert_eq!(result.thinking, "raisonnement affichable");
    assert!(result.continuation.is_none());
}
