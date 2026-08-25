use super::{run, ApiRequestParams};
use crate::services::agent_local::context_usage_buckets::ContextUsageSeed;
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::subagent_orchestration::ParentSubagentOrchestrator;
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::llm::fast_mode;
use crate::services::llm::stream_test_transport::{ScriptedResponse, StreamScenario};
use tokio_util::sync::CancellationToken;

fn message(role: &str, content: String) -> ChatMessage {
    match role {
        "system" => ChatMessage::system(content),
        "user" => ChatMessage::user(content),
        "assistant" => ChatMessage::assistant(content, None, None, None, None),
        "tool" => ChatMessage::tool(content, None, None),
        other => panic!("unsupported chat role in test/setup: {other}"),
    }
}

#[tokio::test]
async fn payload_reduction_retry_keeps_the_generation_fast_capture() {
    let session = crate::services::agent_local::session_store::create_with_project_and_fast_mode(
        "Reduced Fast",
        "gpt-5.6-luna",
        "openai",
        None,
        true,
    )
    .await
    .expect("create session");
    let scenario = StreamScenario::start(
        &session.id,
        [
            ScriptedResponse::PayloadTooLargePaused,
            ScriptedResponse::Success,
        ],
    )
    .await;
    let captured = fast_mode::for_session(&session.id, "openai", "gpt-5.6-luna")
        .await
        .expect("capture generation");
    let emitter = AgentEventEmitter::test(session.id.clone());
    let mut messages = vec![
        message("system", "rules".into()),
        message("user", "a".repeat(16_000)),
        message("assistant", "b".repeat(16_000)),
    ];
    let mut subagents = ParentSubagentOrchestrator::new(&session.id).await;

    let request = run(ApiRequestParams {
        on_event: &emitter,
        provider_id: "openai",
        fast_mode: captured,
        model: "gpt-5.6-luna",
        messages: &mut messages,
        tools: &[],
        think: false,
        reasoning_mode: None,
        session_id: &session.id,
        request_id: "request-payload-reduction",
        cancel: CancellationToken::new(),
        configured_context: 100_000,
        plan_mode_active: false,
        turn: 0,
        subagents: &mut subagents,
        context_usage_seed: ContextUsageSeed::default(),
    });
    let change_preference = async {
        scenario.wait_for_payloads(1).await;
        crate::services::agent_local::session_store::update_fast_mode(&session.id, false)
            .await
            .expect("disable next generation");
        scenario.release_one();
    };
    let (result, ()) = tokio::join!(request, change_preference);
    result.expect("reduced retry succeeds");
    let payloads = scenario.payloads();
    crate::services::agent_local::session_store::delete_one(&session.id)
        .await
        .expect("delete session");

    assert_eq!(payloads.len(), 2);
    assert_eq!(payloads[0]["service_tier"], "fast");
    assert_eq!(payloads[1]["service_tier"], "fast");
    assert!(payloads[1]["input"].as_array().unwrap().len() < 3);
    assert!(payloads[1].get("reasoning_effort").is_none());
}
