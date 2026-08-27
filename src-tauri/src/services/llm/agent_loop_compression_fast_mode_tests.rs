use super::LoopCompression;
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::tool_executor_compression::{
    ToolCompression, ToolCompressionProvider,
};
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::agent_local::types_session::AgentSession;
use crate::services::llm::fast_mode;
use crate::services::llm::stream_test_transport::{ScriptedResponse, StreamScenario};
use tokio_util::sync::CancellationToken;

fn runtime_messages() -> Vec<ChatMessage> {
    vec![
        ChatMessage::system("rules".into()),
        ChatMessage::user("question".into()),
        ChatMessage::assistant("answer".into(), None, None, None, None),
    ]
}

async fn captured_session(name: &str) -> (AgentSession, fast_mode::FastModeRequest) {
    let session = crate::services::agent_local::session_store::create_with_project_and_fast_mode(
        name,
        "gpt-5.6-luna",
        "openai",
        None,
        true,
    )
    .await
    .expect("create session");
    let captured = fast_mode::for_session(&session.id, "openai", "gpt-5.6-luna")
        .await
        .expect("capture generation");
    crate::services::agent_local::session_store::update_fast_mode(&session.id, false)
        .await
        .expect("disable next generation");
    (session, captured)
}

#[tokio::test]
async fn loop_compression_reaches_the_payload_with_the_generation_capture() {
    let (session, captured) = captured_session("Loop compression Fast").await;
    let scenario = StreamScenario::start(&session.id, [ScriptedResponse::Success]).await;
    let emitter = AgentEventEmitter::test(session.id.clone());
    let working_dir = tempfile::tempdir().expect("working directory");
    let compression = LoopCompression {
        on_event: &emitter,
        provider_id: "openai",
        fast_mode: captured,
        model: "gpt-5.6-luna",
        session_id: &session.id,
        request_id: "request-loop-compression",
        native_context: 100_000,
        configured_context: 100_000,
        working_dir: working_dir.path(),
    };
    let mut messages = runtime_messages();

    let result = compression
        .try_run(
            &mut messages,
            Some(90_000),
            Some(0),
            CancellationToken::new(),
        )
        .await;
    let payloads = scenario.payloads();
    crate::services::agent_local::session_store::delete_one(&session.id)
        .await
        .expect("delete session");

    assert!(result.is_some());
    assert_eq!(payloads.len(), 1);
    assert_eq!(payloads[0]["service_tier"], "fast");
    assert!(payloads[0]["input"].is_array());
}

#[tokio::test]
async fn tool_executor_compression_reaches_the_payload_with_the_generation_capture() {
    let (session, captured) = captured_session("Tool compression Fast").await;
    let scenario = StreamScenario::start(&session.id, [ScriptedResponse::Success]).await;
    let emitter = AgentEventEmitter::test(session.id.clone());
    let working_dir = tempfile::tempdir().expect("working directory");
    let compression = ToolCompression {
        on_event: &emitter,
        provider: ToolCompressionProvider::Cloud {
            provider_id: "openai",
            model: "gpt-5.6-luna",
            fast_mode: captured,
        },
        session_id: &session.id,
        request_id: "request-tool-compression",
        native_context: 100_000,
        configured_context: 100_000,
        last_context_tokens: Some(90_000),
        working_dir: working_dir.path(),
        cancel: CancellationToken::new(),
    };
    let mut messages = runtime_messages();

    let result = compression.try_run(&mut messages).await;
    let payloads = scenario.payloads();
    crate::services::agent_local::session_store::delete_one(&session.id)
        .await
        .expect("delete session");

    assert!(result);
    assert_eq!(payloads.len(), 1);
    assert_eq!(payloads[0]["service_tier"], "fast");
    assert!(payloads[0]["input"].is_array());
}
