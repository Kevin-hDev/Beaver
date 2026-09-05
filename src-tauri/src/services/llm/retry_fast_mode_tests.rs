use super::retry_stream;
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::llm::fast_mode::{self, FastModeRequest};
use crate::services::llm::request_purpose::RequestPurpose;
use crate::services::llm::stream_test_transport::{ScriptedResponse, StreamScenario};
use tokio_util::sync::CancellationToken;

#[tokio::test(start_paused = true)]
async fn retryable_provider_failure_is_sent_once_without_idempotency() {
    let session = crate::services::agent_local::session_store::create_with_project_and_fast_mode(
        "Retry Fast",
        "gpt-5.6-luna",
        "openai",
        None,
        true,
    )
    .await
    .expect("create session");
    let scenario = StreamScenario::start(&session.id, [ScriptedResponse::RetryablePaused]).await;
    let captured = fast_mode::for_session(&session.id, "openai", "gpt-5.6-luna")
        .await
        .expect("capture generation");
    let emitter = AgentEventEmitter::test(session.id.clone());
    let messages = [ChatMessage::user("hello".into())];
    let mut next_attempt = 1;
    let previews =
        crate::services::agent_local::tool_artifact_preview::ToolResultPreviewBatch::default();

    let retry = retry_stream(
        &emitter,
        &session.id,
        "request-retry-fast",
        1,
        &mut next_attempt,
        "openai",
        captured,
        RequestPurpose::ManualChat,
        "gpt-5.6-luna",
        &messages,
        &[],
        false,
        None,
        &previews,
        CancellationToken::new(),
        false,
        None,
        None,
    );
    let release_failure = async {
        scenario.wait_for_payloads(1).await;
        scenario.release_one();
    };
    let (result, ()) = tokio::join!(retry, release_failure);

    let payloads = scenario.payloads();
    assert_eq!(result.unwrap_err(), "provider_temporarily_unavailable");
    assert_eq!(payloads.len(), 1);
    assert_eq!(payloads[0]["service_tier"], "fast");
    crate::services::agent_local::session_store::delete_one(&session.id)
        .await
        .expect("delete session");
}

#[tokio::test]
async fn structured_service_tier_refusal_is_sent_once_even_with_tools() {
    let session = crate::services::agent_local::session_store::create_with_project_and_fast_mode(
        "Fast refusal",
        "gpt-5.6-luna",
        "openai",
        None,
        true,
    )
    .await
    .expect("create session");
    let scenario =
        StreamScenario::start(&session.id, [ScriptedResponse::ServiceTierRejected]).await;
    let emitter = AgentEventEmitter::test(session.id.clone());
    let messages = [ChatMessage::user("hello".into())];
    let tools = [serde_json::json!({
        "type": "function",
        "function": {
            "name": "safe_tool",
            "description": "test",
            "parameters": {"type": "object", "properties": {}}
        }
    })];
    let mut next_attempt = 1;
    let previews =
        crate::services::agent_local::tool_artifact_preview::ToolResultPreviewBatch::default();

    let error = retry_stream(
        &emitter,
        &session.id,
        "request-fast-refusal",
        1,
        &mut next_attempt,
        "openai",
        FastModeRequest::Fast,
        RequestPurpose::ManualChat,
        "gpt-5.6-luna",
        &messages,
        &tools,
        false,
        None,
        &previews,
        CancellationToken::new(),
        false,
        None,
        None,
    )
    .await
    .unwrap_err();
    let payloads = scenario.payloads();
    crate::services::agent_local::session_store::delete_one(&session.id)
        .await
        .expect("delete session");

    assert_eq!(error, "service_tier_unavailable");
    assert_eq!(payloads.len(), 1);
    assert_eq!(payloads[0]["service_tier"], "fast");
    assert_eq!(payloads[0]["tools"].as_array().map(Vec::len), Some(1));
}
