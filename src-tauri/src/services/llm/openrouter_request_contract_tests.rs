use super::fast_mode::FastModeRequest;
use super::stream_http::RequestConfig;
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::llm::request_purpose::RequestPurpose;

fn catalog() -> serde_json::Value {
    serde_json::json!({"data":[
        {"id":"vendor/no-tools","architecture":{"output_modalities":["text"]},"supported_parameters":[]},
        {"id":"vendor/tools-only","architecture":{"output_modalities":["text"]},"supported_parameters":["tools","parallel_tool_calls","max_tokens"]},
        {"id":"vendor/choice","architecture":{"output_modalities":["text"]},"supported_parameters":["tools","tool_choice","max_completion_tokens"]},
        {"id":"vendor/fixed","architecture":{"output_modalities":["text"]},"supported_parameters":["reasoning","max_tokens"],"reasoning":{"mandatory":true}},
        {"id":"openai/gpt-6-astra","architecture":{"output_modalities":["text"]},"supported_parameters":["max_tokens"]},
        {"id":"vendor/toggle","architecture":{"output_modalities":["text"]},"supported_parameters":["reasoning"],"reasoning":{"mandatory":false,"default_enabled":false}},
        {"id":"vendor/efforts","architecture":{"output_modalities":["text"]},"supported_parameters":["reasoning"],"reasoning":{"mandatory":false,"supported_efforts":["low","high"],"default_effort":"high"}},
        {"id":"vendor/vision","architecture":{"input_modalities":["text","image"],"output_modalities":["text"]},"supported_parameters":[]},
        {"id":"vendor/free:free","architecture":{"output_modalities":["text"]},"supported_parameters":["max_tokens"],"pricing":{"prompt":"0","completion":"0"}},
        {"id":"vendor/missing","architecture":{"output_modalities":["text"]}}
    ]})
}

async fn publish_catalog() -> Vec<super::types::ModelInfo> {
    let parsed = super::openai_compat_parsing::parse_models_list(&catalog(), "openrouter").unwrap();
    super::model_catalog::enrich_models("openrouter", parsed, false)
        .await
        .unwrap()
}

fn tool() -> serde_json::Value {
    serde_json::json!({
        "type":"function",
        "function":{"name":"search","description":"fixture","parameters":{"type":"object","properties":{}}}
    })
}

fn payload_for(
    model: &str,
    messages: &[ChatMessage],
    tools: &[serde_json::Value],
    think: bool,
    reasoning_mode: Option<&str>,
    max_tokens: Option<u32>,
) -> serde_json::Value {
    let cfg = RequestConfig {
        provider_id: "openrouter",
        model,
        messages,
        tools,
        think,
        reasoning_mode,
        max_tokens,
        purpose: RequestPurpose::ManualChat,
        session_id: None,
        fast_mode: FastModeRequest::Unsupported,
        tool_result_previews: None,
        continuation_target: None,
    };
    super::stream_http_payload::build_chat_payload(
        &cfg,
        &super::route::resolve("openrouter").unwrap(),
        max_tokens,
    )
    .unwrap()
}

#[tokio::test]
async fn parameter_support_is_exact_and_never_inferred_from_a_model_family() {
    let _guard = super::runtime_models::test_mutation_lock().await;
    let models = publish_catalog().await;
    let tools_only = models
        .iter()
        .find(|model| model.id == "vendor/tools-only")
        .unwrap();
    let missing = models
        .iter()
        .find(|model| model.id == "vendor/missing")
        .unwrap();

    assert!(super::openrouter_request_contract::supports_parameter(
        tools_only, "tools"
    ));
    assert!(!super::openrouter_request_contract::supports_parameter(
        tools_only,
        "tool_choice"
    ));
    assert!(!super::openrouter_request_contract::supports_parameter(
        missing, "tools"
    ));
}

#[tokio::test]
async fn gateway_payload_uses_only_published_optional_parameters_before_http() {
    let _guard = super::runtime_models::test_mutation_lock().await;
    publish_catalog().await;
    let tools = [tool()];

    let refused = payload_for("vendor/no-tools", &[], &tools, false, None, Some(2_000));
    assert_eq!(refused["tools"][0]["function"]["name"], "search");
    assert!(refused.get("tool_choice").is_none());
    assert!(refused.get("parallel_tool_calls").is_none());
    assert_eq!(refused["provider"]["require_parameters"], true);

    let tools_only = payload_for("vendor/tools-only", &[], &tools, false, None, Some(2_000));
    assert!(tools_only.get("tool_choice").is_none());
    assert_eq!(tools_only["parallel_tool_calls"], true);
    assert_eq!(tools_only["max_tokens"], 2_000);

    let choice = payload_for("vendor/choice", &[], &tools, false, None, Some(3_000));
    assert_eq!(choice["tool_choice"], "auto");
    assert_eq!(choice["max_completion_tokens"], 3_000);
    assert!(choice.get("max_tokens").is_none());

    for payload in [&refused, &tools_only, &choice] {
        assert!(payload.get("stream_options").is_none());
        for absent in [
            "temperature",
            "top_p",
            "frequency_penalty",
            "presence_penalty",
            "seed",
        ] {
            assert!(payload.get(absent).is_none(), "{absent}");
        }
    }
}

#[tokio::test]
async fn real_http_boundary_records_the_same_gateway_contract_without_network() {
    let _guard = super::runtime_models::test_mutation_lock().await;
    publish_catalog().await;
    let tools = [tool()];
    let cfg = RequestConfig {
        provider_id: "openrouter",
        model: "vendor/tools-only",
        messages: &[],
        tools: &tools,
        think: false,
        reasoning_mode: None,
        max_tokens: Some(2_000),
        purpose: RequestPurpose::ManualChat,
        session_id: Some("openrouter-contract-boundary"),
        fast_mode: FastModeRequest::Unsupported,
        tool_result_previews: None,
        continuation_target: None,
    };
    use super::stream_test_transport::{ScriptedResponse, StreamScenario};
    let scenario =
        StreamScenario::start("openrouter-contract-boundary", [ScriptedResponse::Success]).await;

    super::stream_http::post_chat_request_with_timeout(&cfg, std::time::Duration::from_secs(2))
        .await
        .unwrap();

    let payload = &scenario.payloads()[0];
    assert_eq!(payload["tools"][0]["function"]["name"], "search");
    assert!(payload.get("tool_choice").is_none());
    assert_eq!(payload["parallel_tool_calls"], true);
    assert_eq!(payload["max_tokens"], 2_000);
    assert_eq!(payload["provider"]["require_parameters"], true);
}

#[tokio::test]
async fn reasoning_images_free_variants_and_missing_metadata_keep_their_contracts() {
    let _guard = super::runtime_models::test_mutation_lock().await;
    publish_catalog().await;

    let fixed = payload_for("vendor/fixed", &[], &[], true, Some("auto"), Some(4_000));
    assert!(fixed.get("reasoning").is_none());
    assert_eq!(fixed["max_tokens"], 4_000);
    let astra = payload_for("openai/gpt-6-astra", &[], &[], false, None, Some(512));
    assert_eq!(astra["max_tokens"], 512);
    assert!(astra.get("max_completion_tokens").is_none());
    let toggle = payload_for("vendor/toggle", &[], &[], false, Some("off"), None);
    assert_eq!(toggle["reasoning"], serde_json::json!({"enabled":false}));
    let efforts = payload_for("vendor/efforts", &[], &[], true, Some("low"), None);
    assert_eq!(efforts["reasoning"], serde_json::json!({"effort":"low"}));

    let messages = [ChatMessage::user("describe".into()).with_images(vec!["iVBORw0KGgo=".into()])];
    let vision = payload_for("vendor/vision", &messages, &[], false, None, None);
    assert_eq!(vision["messages"][0]["content"][1]["type"], "image_url");
    let free = payload_for("vendor/free:free", &[], &[], false, None, Some(5_000));
    assert_eq!(free["model"], "vendor/free:free");
    assert_eq!(free["max_tokens"], 5_000);
    let missing = payload_for("vendor/missing", &[], &[], false, None, Some(6_000));
    assert_eq!(missing["max_tokens"], 6_000);
    assert!(missing.get("max_completion_tokens").is_none());
}
