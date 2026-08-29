use crate::services::agent_local::types_ollama::{ChatMessage, ToolCallFunction, ToolCallOllama};
use crate::services::llm::fast_mode::FastModeRequest;
use crate::services::llm::request_purpose::RequestPurpose;
use crate::services::llm::stream_http::RequestConfig;
use serde_json::json;

fn message(role: &str, content: &str) -> ChatMessage {
    ChatMessage {
        continuity_barrier_before: false,
        role: role.to_string(),
        content: content.to_string(),
        images: None,
        tool_calls: None,
        tool_name: None,
        tool_call_id: None,
        display_thinking: None,
        continuation: None,
        tool_loop_reasoning: None,
    }
}

fn config<'a>(
    messages: &'a [ChatMessage],
    tools: &'a [serde_json::Value],
    mode: &'a str,
) -> RequestConfig<'a> {
    config_for_model(messages, tools, "claude-haiku-4-5-20251001", mode)
}

fn config_for_model<'a>(
    messages: &'a [ChatMessage],
    tools: &'a [serde_json::Value],
    model: &'a str,
    mode: &'a str,
) -> RequestConfig<'a> {
    RequestConfig {
        provider_id: "anthropic",
        model,
        messages,
        tools,
        think: mode != "off",
        reasoning_mode: Some(mode),
        max_tokens: None,
        purpose: RequestPurpose::ManualChat,
        session_id: None,
        fast_mode: FastModeRequest::Unsupported,
        continuation_target: None,
    }
}

#[test]
fn payload_uses_native_system_tools_required_limit_and_cache() {
    let messages = vec![message("system", "Be concise"), message("user", "Read it")];
    let tools = vec![json!({
        "type": "function",
        "function": {
            "name": "read_file",
            "description": "Read a file",
            "strict": true,
            "parameters": {"type": "object", "properties": {}}
        }
    })];

    let prepared = super::build_payload(&config(&messages, &tools, "off"), 4_096).unwrap();
    let payload = prepared.payload;

    assert_eq!(payload["max_tokens"], 4_096);
    assert_eq!(payload["system"][0]["type"], "text");
    assert_eq!(payload["tools"][0]["name"], "read_file");
    assert!(payload["tools"][0].get("function").is_none());
    assert!(payload["tools"][0].get("strict").is_none());
    assert_eq!(payload["cache_control"]["type"], "ephemeral");
    for field in ["temperature", "top_p", "top_k"] {
        assert!(payload.get(field).is_none(), "{field}");
    }
}

#[test]
fn tool_results_are_grouped_and_errors_are_marked() {
    let mut assistant = message("assistant", "");
    assistant.tool_calls = Some(vec![
        ToolCallOllama {
            id: Some("call-a".into()),
            extra_content: None,
            function: ToolCallFunction {
                name: "read_file".into(),
                arguments: json!({"path": "a"}),
            },
        },
        ToolCallOllama {
            id: Some("call-b".into()),
            extra_content: None,
            function: ToolCallFunction {
                name: "read_file".into(),
                arguments: json!({"path": "b"}),
            },
        },
    ]);
    let mut success = message("tool", "{\"status\":\"success\"}\nok");
    success.tool_call_id = Some("call-a".into());
    let mut failure = message(
        "tool",
        "{\"kind\":\"tool_result\",\"tool\":\"read_file\",\"status\":\"error\",\"outputFormat\":\"raw_following\"}\nOperation failed",
    );
    failure.tool_call_id = Some("call-b".into());

    let converted = super::messages::convert(&[assistant, success, failure], &[]).unwrap();
    let result = converted.messages.last().unwrap();
    let blocks = result["content"].as_array().unwrap();

    assert_eq!(result["role"], "user");
    assert_eq!(blocks.len(), 2);
    assert_eq!(blocks[0]["type"], "tool_result");
    assert_eq!(blocks[0]["tool_use_id"], "call-a");
    assert_eq!(blocks[0]["is_error"], false);
    assert_eq!(blocks[1]["is_error"], true);

    let mut ordinary = message("tool", "{\"status\":\"error\"}\nLegitimate content");
    ordinary.tool_call_id = Some("call-a".into());
    let ordinary = super::messages::convert(&[ordinary], &[]).unwrap();
    assert_eq!(ordinary.messages[0]["content"][0]["is_error"], false);
}

#[test]
fn image_is_native_base64_before_text() {
    let mut user = message("user", "What is shown?");
    user.images = Some(vec!["iVBORw0KGgo=".into()]);

    let converted = super::messages::convert(&[user], &[]).unwrap();
    let content = converted.messages[0]["content"].as_array().unwrap();

    assert_eq!(content[0]["type"], "image");
    assert_eq!(content[0]["source"]["type"], "base64");
    assert_eq!(content[0]["source"]["media_type"], "image/png");
    assert_eq!(content[0]["source"]["data"], "iVBORw0KGgo=");
    assert_eq!(content[1]["type"], "text");
}

#[test]
fn synthetic_fixture_image_is_one_native_block_and_oversize_fails_before_transport() {
    let mut user = message("user", "Name the quadrants");
    user.images = Some(vec![
        crate::commands::reasoning_fixture_vision::inline_base64().unwrap(),
    ]);
    let converted = super::messages::convert(&[user], &[]).unwrap();
    let content = converted.messages[0]["content"].as_array().unwrap();
    assert_eq!(content.len(), 2);
    assert_eq!(content[0]["type"], "image");
    assert_eq!(content[0]["source"]["media_type"], "image/png");

    let mut oversized = message("user", "image");
    oversized.images = Some(vec![format!(
        "iVBOR{}",
        "A".repeat(
            crate::services::llm::vision::MAX_ANTHROPIC_IMAGE_BYTES.saturating_mul(4) / 3 + 8,
        )
    )]);
    assert_eq!(
        super::messages::convert(&[oversized], &[]),
        Err(super::BuildError::InvalidImage)
    );
}

#[test]
fn thinking_modes_are_bounded_by_output_limit() {
    let messages = vec![message("user", "Hi")];
    let tools = Vec::new();

    for (mode, limit, budget) in [
        ("low", 2_048, 1_024),
        ("medium", 8_192, 4_096),
        ("high", 32_768, 16_384),
    ] {
        assert_eq!(
            super::build_payload(&config(&messages, &tools, mode), limit)
                .unwrap()
                .payload["thinking"]["budget_tokens"],
            budget
        );
    }
    assert!(matches!(
        super::build_payload(&config(&messages, &tools, "medium"), 4_096),
        Err(super::BuildError::InvalidReasoningBudget)
    ));
    assert!(matches!(
        super::build_payload(&config(&messages, &tools, "quantum"), 32_768),
        Err(super::BuildError::InvalidReasoningMode)
    ));
}

#[test]
fn catalog_advertised_adaptive_model_uses_the_validated_anthropic_transport() {
    crate::services::llm::runtime_models::replace_provider(
        "anthropic",
        &[crate::services::llm::types::ModelInfo {
            id: "claude-adaptive-test".into(),
            display_name: None,
            owned_by: Some("anthropic".into()),
            context_length: Some(1_000_000),
            max_output_tokens: Some(128_000),
            supports_tools: true,
            supports_vision: true,
            supports_thinking: true,
            supports_fast_mode: false,
            reasoning_modes: vec!["auto".into(), "low".into(), "xhigh".into()],
            default_reasoning_mode: Some("auto".into()),
            context_usage_includes_reasoning: true,
            is_free: false,
        }],
    );
    let messages = vec![message("user", "Hi")];
    let payload = super::build_payload(
        &config_for_model(&messages, &[], "claude-adaptive-test", "xhigh"),
        128_000,
    )
    .unwrap()
    .payload;

    assert_eq!(payload["thinking"]["type"], "adaptive");
    assert_eq!(payload["output_config"]["effort"], "xhigh");
    assert!(payload["thinking"].get("budget_tokens").is_none());
}

#[test]
fn unknown_anthropic_model_cannot_activate_unvalidated_manual_thinking() {
    let messages = vec![message("user", "Hi")];
    let payload = super::build_payload(
        &config_for_model(&messages, &[], "claude-unknown", "high"),
        32_000,
    )
    .unwrap()
    .payload;

    assert_eq!(payload["thinking"]["type"], "disabled");
    assert!(payload["thinking"].get("budget_tokens").is_none());
}

#[test]
fn rejects_invalid_images_tool_schemas_and_limits() {
    let mut invalid_image = message("user", "image");
    invalid_image.images = Some(vec!["not-base64".into()]);
    assert_eq!(
        super::messages::convert(&[invalid_image], &[]),
        Err(super::BuildError::InvalidImage)
    );

    let malformed = vec![json!({"type": "function", "function": {"name": "broken"}})];
    assert_eq!(
        super::tools::convert(&malformed),
        Err(super::BuildError::InvalidToolSchema)
    );
    let messages = vec![message("user", "Hi")];
    assert!(matches!(
        super::build_payload(&config(&messages, &[], "off"), 0),
        Err(super::BuildError::InvalidMaxTokens)
    ));
}
