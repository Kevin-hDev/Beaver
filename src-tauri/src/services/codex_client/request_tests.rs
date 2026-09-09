use super::*;
use crate::services::llm::fast_mode::FastModeRequest;

#[test]
fn codex_request_keeps_only_model_supported_effort() {
    let sol = build_codex_request(
        "gpt-5.6-sol",
        &[],
        &[],
        Some("ultra"),
        None,
        FastModeRequest::Standard,
    );
    let luna = build_codex_request(
        "gpt-5.6-luna",
        &[],
        &[],
        Some("ultra"),
        None,
        FastModeRequest::Standard,
    );

    assert_eq!(sol.reasoning.unwrap().effort, "ultra");
    assert_eq!(luna.reasoning.unwrap().effort, "medium");
}

#[tokio::test]
async fn astra_request_uses_catalog_effort_and_responses_media_contract() {
    let _guard = crate::services::llm::runtime_models::test_mutation_lock().await;
    crate::services::llm::runtime_models::replace_provider(
        "codex-oauth",
        &[crate::services::llm::types::ModelInfo {
            id: "gpt-6-astra".into(),
            display_name: Some("GPT-6 Astra".into()),
            owned_by: Some("openai".into()),
            context_length: Some(1_050_000),
            max_output_tokens: Some(128_000),
            supports_tools: true,
            supports_vision: true,
            supports_thinking: true,
            reasoning_metadata_present: false,
            supports_fast_mode: false,
            reasoning_modes: vec![
                "low".into(),
                "medium".into(),
                "high".into(),
                "xhigh".into(),
                "max".into(),
            ],
            default_reasoning_mode: Some("medium".into()),
            context_usage_includes_reasoning: true,
            is_free: false,
        }],
    )
    .unwrap();
    let messages = [
        crate::services::agent_local::types_ollama::ChatMessage::user("describe".into())
            .with_images(vec!["iVBORw0KGgo=".into()]),
    ];
    let tools = [serde_json::json!({
        "type": "function",
        "function": {
            "name": "search",
            "description": "fixture",
            "parameters": {"type": "object", "properties": {}}
        }
    })];

    let request = build_codex_request(
        "gpt-6-astra",
        &messages,
        &tools,
        Some("max"),
        None,
        FastModeRequest::Unsupported,
    );
    let body = serde_json::to_value(request).unwrap();
    assert_eq!(
        body["reasoning"],
        serde_json::json!({"effort": "max", "summary": "auto"})
    );
    assert_eq!(body["store"], false);
    assert_eq!(body["input"][0]["content"][1]["type"], "input_image");
    assert_eq!(body["tools"][0]["type"], "function");
    assert_eq!(
        body["include"],
        serde_json::json!(["reasoning.encrypted_content"])
    );
    for forbidden in ["all_turns", "sampling", "thinking", "clear_thinking"] {
        assert!(body.get(forbidden).is_none(), "unexpected {forbidden}");
    }

    crate::services::llm::runtime_models::replace_provider("codex-oauth", &[]).unwrap();
}

#[test]
fn request_keeps_the_official_empty_tools_contract() {
    let request = build_codex_request(
        "gpt-5.6-sol",
        &[],
        &[],
        None,
        Some("session-1"),
        FastModeRequest::Standard,
    );
    let json = serde_json::to_value(request).unwrap();

    assert_eq!(json["tools"], serde_json::json!([]));
    assert_eq!(json["tool_choice"], "auto");
    assert!(json["prompt_cache_key"]
        .as_str()
        .is_some_and(|value| value.starts_with("bv1_")));
}

#[test]
fn codex_request_and_routing_hint_share_the_captured_fast_mode() {
    let fast = build_codex_request("gpt-5.6-sol", &[], &[], None, None, FastModeRequest::Fast);
    let standard = build_codex_request(
        "gpt-5.6-sol",
        &[],
        &[],
        None,
        None,
        FastModeRequest::Standard,
    );
    let unsupported = build_codex_request(
        "gpt-5.4-mini",
        &[],
        &[],
        None,
        None,
        FastModeRequest::Unsupported,
    );

    assert_eq!(
        serde_json::to_value(&fast).unwrap()["service_tier"],
        "priority"
    );
    assert!(serde_json::to_value(&standard)
        .unwrap()
        .get("service_tier")
        .is_none());
    assert!(serde_json::to_value(&unsupported)
        .unwrap()
        .get("service_tier")
        .is_none());
    assert_eq!(
        crate::services::codex_client::routing_hint::for_request(&fast).unwrap(),
        "model=gpt-5.6-sol;tier=priority"
    );
    assert_eq!(
        crate::services::codex_client::routing_hint::for_request(&standard).unwrap(),
        "model=gpt-5.6-sol"
    );
    assert_eq!(
        crate::services::codex_client::routing_hint::for_request(&unsupported).unwrap(),
        "model=gpt-5.4-mini"
    );
}

#[test]
fn routing_hint_rejects_untrusted_model_or_tier_and_stays_bounded() {
    let mut invalid_model = build_codex_request(
        "gpt-5.6-sol",
        &[],
        &[],
        None,
        None,
        FastModeRequest::Standard,
    );
    invalid_model.model = "../outside".to_string();
    assert_eq!(
        crate::services::codex_client::routing_hint::for_request(&invalid_model).unwrap_err(),
        "provider_configuration_invalid"
    );

    let mut invalid_tier =
        build_codex_request("gpt-5.6-sol", &[], &[], None, None, FastModeRequest::Fast);
    invalid_tier.service_tier = Some("default".to_string());
    assert_eq!(
        crate::services::codex_client::routing_hint::for_request(&invalid_tier).unwrap_err(),
        "provider_configuration_invalid"
    );

    let longest_valid_model = "a".repeat(128);
    let request = build_codex_request(
        &longest_valid_model,
        &[],
        &[],
        None,
        None,
        FastModeRequest::Fast,
    );
    let hint = crate::services::codex_client::routing_hint::for_request(&request).unwrap();
    assert!(hint.len() <= 160);
    assert_eq!(hint, format!("model={longest_valid_model};tier=priority"));
}

#[tokio::test]
async fn cancellation_is_observed_before_response_headers() {
    let cancel = CancellationToken::new();
    cancel.cancel();

    let result = cancel_aware(&cancel, std::future::pending::<Result<(), String>>()).await;

    assert_eq!(result.unwrap_err(), "Annulé");
}
