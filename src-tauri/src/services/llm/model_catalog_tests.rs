use super::types::ModelInfo;

fn remote_anthropic_model() -> ModelInfo {
    remote_anthropic_model_with_id("claude-haiku-4-5-20251001")
}

fn remote_anthropic_model_with_id(id: &str) -> ModelInfo {
    ModelInfo {
        id: id.into(),
        display_name: Some(id.into()),
        owned_by: Some("anthropic".into()),
        context_length: Some(180_000),
        max_output_tokens: Some(32_000),
        supports_tools: false,
        supports_vision: true,
        supports_thinking: true,
        supports_fast_mode: false,
        reasoning_modes: vec!["off".into(), "low".into()],
        default_reasoning_mode: Some("low".into()),
        context_usage_includes_reasoning: true,
        is_free: false,
    }
}

fn remote_qwen_model(id: &str) -> ModelInfo {
    ModelInfo {
        id: id.into(),
        display_name: Some(id.into()),
        owned_by: Some("qwen".into()),
        context_length: None,
        max_output_tokens: None,
        supports_tools: false,
        supports_vision: false,
        supports_thinking: false,
        supports_fast_mode: false,
        reasoning_modes: Vec::new(),
        default_reasoning_mode: None,
        context_usage_includes_reasoning: true,
        is_free: false,
    }
}

#[tokio::test]
async fn native_catalog_keeps_explicit_remote_values() {
    let models =
        super::model_catalog::enrich_models("anthropic", vec![remote_anthropic_model()], true)
            .await
            .unwrap();

    assert_eq!(models.len(), 1);
    assert_eq!(models[0].context_length, Some(180_000));
    assert_eq!(models[0].max_output_tokens, Some(32_000));
    assert!(!models[0].supports_tools);
    assert_eq!(models[0].default_reasoning_mode.as_deref(), Some("low"));
}

#[tokio::test]
async fn catalog_is_deduplicated_before_runtime_registration() {
    let model = remote_anthropic_model();
    let models = super::model_catalog::enrich_models("anthropic", vec![model.clone(), model], true)
        .await
        .unwrap();

    assert_eq!(models.len(), 1);
    assert!(super::runtime_models::lookup("anthropic", "claude-haiku-4-5-20251001").is_some());
}

#[tokio::test]
async fn qwen_remote_catalog_keeps_every_chat_model_returned_by_the_account() {
    let models = super::model_catalog::enrich_models(
        "qwen",
        vec![
            remote_qwen_model("qwen3.8-max"),
            remote_qwen_model("qwen3.8-flash"),
        ],
        false,
    )
    .await
    .unwrap();

    assert_eq!(models.len(), 2);
    assert_eq!(models[0].id, "qwen3.8-max");
    assert_eq!(models[1].id, "qwen3.8-flash");
    assert_eq!(models[0].context_length, Some(1_000_000));
    assert_eq!(models[0].max_output_tokens, Some(131_072));
    assert!(models[0].supports_tools);
    assert!(models[0].supports_vision);
    assert!(models[0].supports_thinking);
    assert_eq!(models[0].reasoning_modes, ["off", "low", "medium", "xhigh"]);
    assert!(models[1].supports_tools);
    assert!(models[1].supports_vision);
    assert!(models[1].supports_thinking);
    assert_eq!(models[1].reasoning_modes, ["off", "low", "medium", "xhigh"]);
}

#[tokio::test]
async fn qwen_successful_catalog_without_the_test_model_stays_usable() {
    let models =
        super::model_catalog::enrich_models("qwen", vec![remote_qwen_model("qwen3.8-max")], false)
            .await
            .unwrap();

    assert_eq!(models.len(), 1);
    assert_eq!(models[0].id, "qwen3.8-max");
    assert!(models[0].supports_tools);
    assert!(models[0].supports_vision);
}

#[tokio::test]
async fn anthropic_catalog_keeps_the_reasoning_modes_advertised_by_the_model() {
    let models = super::model_catalog::enrich_models(
        "anthropic",
        vec![remote_anthropic_model_with_id("claude-sonnet-5")],
        true,
    )
    .await
    .unwrap();

    assert_eq!(models.len(), 1);
    assert_eq!(models[0].id, "claude-sonnet-5");
    assert!(models[0].supports_vision);
    assert!(models[0].supports_thinking);
    assert_eq!(models[0].reasoning_modes, ["off", "low"]);
    assert_eq!(models[0].default_reasoning_mode.as_deref(), Some("low"));
}

#[tokio::test]
async fn provider_transport_proofs_do_not_restrict_third_party_catalog_modes() {
    let mut model = remote_anthropic_model_with_id("gpt-5.5");
    model.owned_by = Some("openai".into());
    model.reasoning_modes = vec!["off".into(), "low".into(), "high".into()];
    model.default_reasoning_mode = Some("high".into());

    let models = super::model_catalog::enrich_models("openai", vec![model], true)
        .await
        .unwrap();

    assert!(models[0].supports_thinking);
    assert_eq!(models[0].reasoning_modes, ["off", "low", "high"]);
    assert_eq!(models[0].default_reasoning_mode.as_deref(), Some("high"));
}
