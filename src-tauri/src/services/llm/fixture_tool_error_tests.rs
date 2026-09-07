//! P06 offline: a real fixture failure reaches the provider's tool-result slot.
//! This is not an account-access or reasoning-continuity proof.

use super::{fast_mode::FastModeRequest, request_purpose::RequestPurpose};
use crate::services::agent_local::{
    fixture_tool_executor,
    stream_events::AgentEventEmitter,
    types_ollama::{ChatMessage, OllamaThink, ToolCallFunction, ToolCallOllama},
};
use crate::services::reasoning_fixture_run::FixtureRunContext;
use serde_json::{json, Value};
use tokio_util::sync::CancellationToken;

const CALL_ID: &str = "call_fixture_error";
const TOOL: &str = "fixture.read_note";

async fn failed_tool_history() -> (Vec<ChatMessage>, [Value; 2]) {
    let mut run = FixtureRunContext::start().await.unwrap();
    let root = run.root_for_test();
    let tools = run.definitions();
    let mut messages = vec![ChatMessage::assistant(
        String::new(),
        None,
        None,
        None,
        Some(vec![ToolCallOllama {
            id: Some(CALL_ID.into()),
            extra_content: None,
            function: ToolCallFunction {
                name: TOOL.into(),
                arguments: json!({}),
            },
        }]),
    )];
    // Reading before writing exercises a real missing-file error, not a mock.
    let mut outcome = fixture_tool_executor::execute(
        &AgentEventEmitter::test("fixture-error-p06".into()),
        &mut messages,
        &[(TOOL.into(), json!({}))],
        &[CALL_ID.into()],
        &mut run,
        &CancellationToken::new(),
    )
    .await;
    assert!(!outcome.apply_follow_ups(&mut messages).unwrap());
    assert_eq!(messages.len(), 2);
    let result = &messages[1];
    assert_eq!(result.role, "tool");
    assert_eq!(result.tool_call_id.as_deref(), Some(CALL_ID));
    assert_eq!(result.tool_name.as_deref(), Some(TOOL));
    let (header, content) = result.content.split_once('\n').unwrap();
    let header: Value = serde_json::from_str(header).unwrap();
    assert_eq!(header["status"], "error");
    assert_eq!(header["error"]["code"], "fixture_tool_unavailable");
    assert_eq!(content, "Outil de fixture indisponible.");
    assert!(!serde_json::to_string(&messages)
        .unwrap()
        .contains(root.to_str().unwrap()));
    drop(run);
    assert!(!root.exists());
    (messages, tools)
}

fn config<'a>(
    provider_id: &'a str,
    model: &'a str,
    messages: &'a [ChatMessage],
    tools: &'a [Value],
) -> super::stream_http::RequestConfig<'a> {
    super::stream_http::RequestConfig {
        provider_id,
        model,
        messages,
        tools,
        think: false,
        reasoning_mode: None,
        max_tokens: Some(64),
        purpose: RequestPurpose::ManualChat,
        session_id: None,
        fast_mode: FastModeRequest::Standard,
        tool_result_previews: None,
        continuation_target: None,
    }
}

#[tokio::test]
async fn fixture_error_keeps_call_id_in_chat_compatible_payloads() {
    let (messages, tools) = failed_tool_history().await;
    for (provider, model) in [
        ("google", "gemini-3.8-flash"),
        ("zai", "glm-5.3-flash"),
        ("openrouter", "google/gemini-3.8-flash"),
        ("openrouter", "z-ai/glm-5.3-flash"),
        ("openrouter", "openai/gpt-6-astra"),
        // Generic tool placement only; Alibaba's exact reasoning contract is blocked.
        ("qwen", "ZHIPU/GLM-5.3-Flash"),
    ] {
        let payload = super::stream_http_payload::build_chat_payload(
            &config(provider, model, &messages, &tools),
            &super::route::test_route(provider),
            Some(64),
        )
        .unwrap();
        assert_eq!(payload["messages"][0]["tool_calls"][0]["id"], CALL_ID);
        assert_eq!(payload["messages"][1]["role"], "tool");
        assert_eq!(payload["messages"][1]["tool_call_id"], CALL_ID);
        assert_eq!(payload["messages"][1]["content"], messages[1].content);
    }
}

#[tokio::test]
async fn fixture_error_keeps_call_id_in_responses_items() {
    let (messages, tools) = failed_tool_history().await;
    let direct =
        super::openai_responses::build_request(&config("openai", "gpt-6-astra", &messages, &tools));
    let (_, codex) =
        crate::services::codex_client::convert::convert_messages_with_tools(&messages, &tools);
    for input in [direct["input"].as_array().unwrap(), &codex] {
        assert_eq!(input.len(), 2);
        assert_eq!(input[0]["type"], "function_call");
        assert_eq!(input[0]["call_id"], CALL_ID);
        assert_eq!(input[1]["type"], "function_call_output");
        assert_eq!(input[1]["call_id"], CALL_ID);
        assert_eq!(input[1]["output"], messages[1].content);
    }
}

#[tokio::test]
async fn fixture_error_keeps_native_tool_name_and_order() {
    use crate::services::agent_local::{agent_loop_support, ollama_tool_role, ollama_wire};
    let (messages, tools) = failed_tool_history().await;
    let request = agent_loop_support::build_request(
        "glm-5.3-flash:cloud",
        &messages,
        &tools,
        OllamaThink::Bool(false),
    );
    let wire_messages = ollama_tool_role::wrap_tool_results(
        &messages,
        super::route_profile::ToolResultPlacement::OllamaNative,
    );
    let body = ollama_wire::chat_request(&request, &wire_messages).unwrap();
    assert_eq!(
        body["messages"][0]["tool_calls"][0]["function"]["name"],
        TOOL
    );
    assert_eq!(body["messages"][1]["role"], "user");
    assert_eq!(
        body["messages"][1]["content"],
        format!(
            "<tool_response> name=\"fixture.read_note\"\n{}\n</tool_response>",
            messages[1].content,
        )
    );
    assert_eq!(messages[1].role, "tool");
}
