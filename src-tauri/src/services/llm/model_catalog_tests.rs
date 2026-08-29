use super::types::ModelInfo;

fn remote_anthropic_model() -> ModelInfo {
    ModelInfo {
        id: "claude-haiku-4-5-20251001".into(),
        display_name: Some("Remote Haiku".into()),
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
