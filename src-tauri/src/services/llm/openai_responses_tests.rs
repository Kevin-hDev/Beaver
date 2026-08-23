use super::build_request;
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::llm::fast_mode::FastModeRequest;
use crate::services::llm::request_purpose::RequestPurpose;
use crate::services::llm::stream_http::RequestConfig;

fn request<'a>(
    messages: &'a [ChatMessage],
    tools: &'a [serde_json::Value],
    reasoning_mode: Option<&'a str>,
    fast_mode: FastModeRequest,
) -> RequestConfig<'a> {
    RequestConfig {
        provider_id: "openai",
        model: "gpt-5.6-luna",
        messages,
        tools,
        think: reasoning_mode != Some("off"),
        reasoning_mode,
        max_tokens: None,
        purpose: RequestPurpose::ManualChat,
        session_id: Some("session-fixture"),
        fast_mode,
    }
}

#[test]
fn api_request_uses_responses_reasoning_and_fast_contract() {
    let messages = [ChatMessage {
        role: "user".into(),
        content: "bonjour".into(),
        ..Default::default()
    }];
    let body = build_request(&request(
        &messages,
        &[],
        Some("medium"),
        FastModeRequest::Fast,
    ));

    assert_eq!(body["model"], "gpt-5.6-luna");
    assert_eq!(body["reasoning"]["effort"], "medium");
    assert_eq!(body["reasoning"]["summary"], "auto");
    assert_eq!(body["service_tier"], "fast");
    assert_eq!(body["stream"], true);
    assert_eq!(body["store"], false);
    assert!(body.get("reasoning_effort").is_none());
    assert!(body.get("messages").is_none());
}

#[test]
fn api_request_preserves_tool_continuation_in_responses_shape() {
    let messages = [
        ChatMessage {
            role: "assistant".into(),
            tool_calls: Some(vec![
                crate::services::agent_local::types_ollama::ToolCallOllama {
                    id: Some("call_1".into()),
                    function: crate::services::agent_local::types_ollama::ToolCallFunction {
                        name: "lookup".into(),
                        arguments: serde_json::json!({"city": "Paris"}),
                    },
                    extra_content: None,
                },
            ]),
            ..Default::default()
        },
        ChatMessage {
            role: "tool".into(),
            content: "18 C".into(),
            tool_call_id: Some("call_1".into()),
            ..Default::default()
        },
    ];
    let tools = [serde_json::json!({
        "type": "function",
        "function": {
            "name": "lookup",
            "description": "Lookup weather",
            "parameters": {"type": "object", "properties": {}}
        }
    })];
    let body = build_request(&request(
        &messages,
        &tools,
        Some("high"),
        FastModeRequest::Standard,
    ));

    assert_eq!(body["service_tier"], "default");
    assert_eq!(body["tools"][0]["type"], "function");
    assert_eq!(body["input"][0]["type"], "function_call");
    assert_eq!(body["input"][1]["type"], "function_call_output");
}

#[test]
fn unsupported_fast_mode_omits_service_tier() {
    let messages = [ChatMessage {
        role: "user".into(),
        content: "bonjour".into(),
        ..Default::default()
    }];
    let body = build_request(&request(
        &messages,
        &[],
        Some("medium"),
        FastModeRequest::Unsupported,
    ));

    assert!(body.get("service_tier").is_none());
}

#[tokio::test]
async fn runtime_dispatch_cannot_fall_back_to_chat_completions() {
    let session_id = "openai-responses-runtime";
    let scenario = crate::services::llm::stream_test_transport::StreamScenario::start(
        session_id,
        [crate::services::llm::stream_test_transport::ScriptedResponse::Success],
    )
    .await;
    let emitter = crate::services::agent_local::stream_events::AgentEventEmitter::test(
        session_id.to_string(),
    );
    let messages = [ChatMessage {
        role: "user".into(),
        content: "bonjour".into(),
        ..Default::default()
    }];

    crate::services::llm::stream::stream_chat_no_done(
        &emitter,
        session_id,
        "request-responses-runtime",
        0,
        1,
        "openai",
        FastModeRequest::Fast,
        RequestPurpose::ManualChat,
        "gpt-5.6-luna",
        &messages,
        &[],
        true,
        Some("medium"),
        tokio_util::sync::CancellationToken::new(),
        false,
        None,
    )
    .await
    .expect("Responses stream completes");

    let payloads = scenario.payloads();
    assert_eq!(payloads.len(), 1);
    assert_eq!(payloads[0]["reasoning"]["effort"], "medium");
    assert_eq!(payloads[0]["service_tier"], "fast");
    assert!(payloads[0].get("reasoning_effort").is_none());
    assert!(payloads[0].get("messages").is_none());
}
