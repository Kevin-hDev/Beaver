use crate::services::agent_local::types_ollama::{ChatMessage, StreamOutcome};
use crate::services::llm::fast_mode::FastModeRequest;
use crate::services::llm::request_purpose::RequestPurpose;
use crate::services::llm::stream_http::RequestConfig;
use crate::services::llm::stream_test_transport::{ScriptedResponse, StreamScenario};
use tokio_util::sync::CancellationToken;

fn config<'a>(messages: &'a [ChatMessage], session_id: &'a str) -> RequestConfig<'a> {
    RequestConfig {
        provider_id: "anthropic",
        model: "claude-haiku-4-5-20251001",
        messages,
        tools: &[],
        think: false,
        reasoning_mode: Some("off"),
        max_tokens: Some(256),
        purpose: RequestPurpose::ManualChat,
        session_id: Some(session_id),
        fast_mode: FastModeRequest::Unsupported,
        tool_result_previews: None,
        continuation_target: None,
    }
}

#[tokio::test]
async fn interactive_transport_posts_native_messages_and_consumes_native_sse() {
    let scenario =
        StreamScenario::start("anthropic-interactive", [ScriptedResponse::Success]).await;
    let messages = [ChatMessage::user("Reply briefly".into())];
    let outcome = super::stream_chat(
        &crate::services::agent_local::stream_events::AgentEventEmitter::test("session".into()),
        &config(&messages, "anthropic-interactive"),
        CancellationToken::new(),
        true,
        None,
        None,
        "request-1",
        None,
    )
    .await
    .unwrap();
    let StreamOutcome::Completed(result) = outcome else {
        panic!("completed Anthropic response")
    };

    assert_eq!(result.content, "ok");
    let payloads = scenario.payloads();
    assert_eq!(payloads[0]["model"], "claude-haiku-4-5-20251001");
    assert_eq!(payloads[0]["messages"][0]["role"], "user");
    assert_eq!(payloads[0]["max_tokens"], 256);
    assert_eq!(payloads[0]["thinking"]["type"], "disabled");
}

#[tokio::test]
async fn silent_transport_uses_requested_limit_without_tools_or_thinking() {
    let scenario = StreamScenario::start("anthropic-silent", [ScriptedResponse::Success]).await;
    let messages = [ChatMessage::user("Summarize".into())];
    let result = super::collect_silent(
        &config(&messages, "anthropic-silent"),
        CancellationToken::new(),
        None,
    )
    .await
    .unwrap();

    assert_eq!(result.content, "ok");
    let payloads = scenario.payloads();
    assert_eq!(payloads[0]["max_tokens"], 256);
    assert_eq!(payloads[0]["thinking"]["type"], "disabled");
    assert!(payloads[0].get("tools").is_none());
}
