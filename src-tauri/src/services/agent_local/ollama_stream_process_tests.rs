use super::ollama_stream_process::{done_generation_duration, process_chunk, ProcessChunkOptions};
use crate::services::agent_local::agent_loop_support::build_assistant_message;
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::StreamResult;
use crate::services::llm::reasoning_wire::{ReasoningCapture, ReasoningCaptureContext};
use crate::services::reasoning_continuity::contract::{CredentialScope, ReasoningModeId, RouteId};
use crate::services::stream_utils::ThinkTagFilter;

fn replay_text_fragments(
    mut fragments: crate::services::llm::stream_fragments::StreamFragmentState,
    chunks: &[&str],
) -> StreamResult {
    let mut result = StreamResult::default();
    let mut token_count = 0;
    let mut filter = ThinkTagFilter::new();
    let emitter = AgentEventEmitter::test("session".into());
    for chunk in chunks {
        process_chunk(
            chunk,
            &emitter,
            &mut token_count,
            &mut result,
            None,
            &mut filter,
            ProcessChunkOptions {
                buffer_content: true,
                reasoning_capture: None,
                fragments: &mut fragments,
            },
        )
        .expect("valid Ollama fixture");
    }
    result
}

#[test]
fn ollama_differential_and_cumulative_fixtures_produce_the_same_text() {
    let differential = replay_text_fragments(
        crate::services::llm::stream_fragments::StreamFragmentState::ollama(),
        &[
            r#"{"message":{"thinking":"rai","content":"Bon"},"done":false}"#,
            r#"{"message":{"thinking":"son","content":"jour"},"done":false}"#,
        ],
    );
    let cumulative = replay_text_fragments(
        crate::services::llm::stream_fragments::StreamFragmentState::cumulative_fixture(),
        &[
            r#"{"message":{"thinking":"rai","content":"Bon"},"done":false}"#,
            r#"{"message":{"thinking":"raison","content":"Bonjour"},"done":false}"#,
        ],
    );

    assert_eq!(differential.content, "Bonjour");
    assert_eq!(differential.thinking, "raison");
    assert_eq!(cumulative.content, differential.content);
    assert_eq!(cumulative.thinking, differential.thinking);
}

#[test]
fn reads_bounded_native_ollama_generation_duration() {
    let chunk = serde_json::json!({ "eval_duration": 2_500_000_000_u64 });

    assert_eq!(done_generation_duration(&chunk), Some(2_500_000_000));
}

#[test]
fn rejects_invalid_native_ollama_generation_duration() {
    assert_eq!(
        done_generation_duration(&serde_json::json!({ "eval_duration": 0 })),
        None
    );
}

#[test]
fn disabled_capture_does_not_create_a_continuation_envelope() {
    let mut result = StreamResult::default();
    let mut token_count = 0;
    let mut filter = ThinkTagFilter::new();
    let mut fragments = crate::services::llm::stream_fragments::StreamFragmentState::ollama();
    let emitter = AgentEventEmitter::test("session".into());

    process_chunk(
        r#"{"message":{"thinking":"raisonnement affichable"},"done":false}"#,
        &emitter,
        &mut token_count,
        &mut result,
        None,
        &mut filter,
        ProcessChunkOptions {
            buffer_content: true,
            reasoning_capture: None,
            fragments: &mut fragments,
        },
    )
    .unwrap();
    process_chunk(
        r#"{"done":true,"done_reason":"stop"}"#,
        &emitter,
        &mut token_count,
        &mut result,
        None,
        &mut filter,
        ProcessChunkOptions {
            buffer_content: true,
            reasoning_capture: None,
            fragments: &mut fragments,
        },
    )
    .unwrap();

    assert_eq!(result.thinking, "raisonnement affichable");
    assert!(result.continuation.is_none());
}

#[test]
fn native_ollama_tool_calls_receive_unique_local_ids_aligned_with_the_journal() {
    let mut result = StreamResult::default();
    let mut token_count = 0;
    let mut filter = ThinkTagFilter::new();
    let mut fragments = crate::services::llm::stream_fragments::StreamFragmentState::ollama();
    let emitter = AgentEventEmitter::test("session".into());

    process_chunk(
        r#"{"message":{"tool_calls":[
            {"function":{"name":"fixture.write_note","arguments":{"value":"first"}}},
            {"function":{"name":"fixture.read_note","arguments":{}}}
        ]},"done":false}"#,
        &emitter,
        &mut token_count,
        &mut result,
        None,
        &mut filter,
        ProcessChunkOptions {
            buffer_content: true,
            reasoning_capture: None,
            fragments: &mut fragments,
        },
    )
    .unwrap();

    assert_eq!(result.tool_calls.len(), 2);
    assert_eq!(result.tool_call_ids.len(), result.tool_calls.len());
    assert_ne!(result.tool_call_ids[0], result.tool_call_ids[1]);
    for id in &result.tool_call_ids {
        assert_eq!(uuid::Uuid::parse_str(id).unwrap().get_version_num(), 4);
    }

    let assistant = build_assistant_message(&result);
    let calls = assistant.tool_calls.expect("journal assistant calls");
    assert_eq!(calls.len(), result.tool_call_ids.len());
    for (call, id) in calls.iter().zip(&result.tool_call_ids) {
        assert_eq!(call.id.as_deref(), Some(id.as_str()));
    }
}

#[test]
fn native_ollama_capture_links_local_tool_ids_for_next_turn_admission() {
    let mut result = StreamResult::default();
    let mut token_count = 0;
    let mut filter = ThinkTagFilter::new();
    let mut fragments = crate::services::llm::stream_fragments::StreamFragmentState::ollama();
    let emitter = AgentEventEmitter::test("session".into());
    let mut capture = ReasoningCapture::new(ReasoningCaptureContext {
        route_id: RouteId::Ollama,
        model_id: "qwen3.5:4b".into(),
        credential_scope: CredentialScope::local_uncredentialed(),
        reasoning_mode: ReasoningModeId::Auto,
    })
    .expect("capture");

    process_chunk(
        r#"{"message":{"thinking":"opaque","tool_calls":[{"function":{"name":"fixture.write_note","arguments":{}}}]},"done":false}"#,
        &emitter,
        &mut token_count,
        &mut result,
        None,
        &mut filter,
        ProcessChunkOptions {
            buffer_content: true,
            reasoning_capture: Some(&mut capture),
            fragments: &mut fragments,
        },
    )
    .expect("tool chunk");
    process_chunk(
        r#"{"done":true,"done_reason":"stop"}"#,
        &emitter,
        &mut token_count,
        &mut result,
        None,
        &mut filter,
        ProcessChunkOptions {
            buffer_content: true,
            reasoning_capture: Some(&mut capture),
            fragments: &mut fragments,
        },
    )
    .expect("done chunk");

    let envelope = result.continuation.expect("persisted continuation");
    assert_eq!(envelope.tool_links.len(), 1);
    assert_eq!(
        envelope.tool_links[0].provider_call_id,
        result.tool_call_ids[0]
    );
    assert_eq!(envelope.tool_links[0].tool_name, result.tool_calls[0].0);
}
