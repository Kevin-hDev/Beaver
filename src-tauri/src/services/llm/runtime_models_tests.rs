use super::*;

fn model(id: String) -> ModelInfo {
    ModelInfo {
        id,
        display_name: None,
        owned_by: None,
        context_length: Some(256_000),
        max_output_tokens: Some(64_000),
        supported_parameters: None,
        catalog_capabilities: Default::default(),
        supports_tools: true,
        supports_vision: true,
        supports_thinking: true,
        reasoning_contract:
            crate::services::llm::model_reasoning_contract::ModelReasoningContract::from_names(
                &["auto"],
                Some("auto"),
            ),
        supports_fast_mode: false,
        context_usage_includes_reasoning: true,
        is_free: true,
    }
}

#[tokio::test]
async fn catalog_rows_preserve_native_runtime_limit_policy() {
    let _guard = test_mutation_lock().await;
    replace_provider("moonshot", &[model("stable".to_string())]).unwrap();
    let models = (0..600)
        .map(|index| model(format!("kimi-{index}")))
        .collect::<Vec<_>>();
    replace_provider("moonshot", &models).unwrap();
    assert!(lookup("moonshot", "kimi-0").is_some());
    assert!(lookup("moonshot", "kimi-499").is_some());
    assert!(lookup("moonshot", "kimi-500").is_none());
    replace_provider(
        "moonshot",
        &[model("../invalid".to_string()), model("stable".to_string())],
    )
    .unwrap();
    assert!(lookup("moonshot", "stable").is_some());
    assert!(lookup("moonshot", "../invalid").is_none());

    let mut invalid_reasoning = model("invalid-reasoning".to_string());
    invalid_reasoning.reasoning_contract = Some(
        crate::services::llm::model_reasoning_contract::ModelReasoningContract {
            mandatory: Some(true),
            default_enabled: Some(true),
            supports_max_tokens: None,
            default_effort: Some(
                crate::services::reasoning_continuity::contract::ReasoningModeId::High,
            ),
            control: crate::services::llm::model_reasoning_contract::ReasoningControl::Efforts(
                vec![crate::services::reasoning_continuity::contract::ReasoningModeId::Low],
            ),
        },
    );
    replace_provider(
        "moonshot",
        &[invalid_reasoning, model("stable".to_string())],
    )
    .unwrap();
    assert!(lookup("moonshot", "stable").is_some());
    assert!(lookup("moonshot", "invalid-reasoning").is_none());
}

#[tokio::test]
async fn duplicate_ids_are_rejected_without_replacing_the_registry() {
    let _guard = test_mutation_lock().await;
    replace_provider("openrouter", &[model("stable".to_string())]).unwrap();
    let duplicate = model("duplicate".to_string());

    assert_eq!(
        replace_provider("openrouter", &[duplicate.clone(), duplicate]),
        Err("duplicate_model_id")
    );
    assert!(lookup("openrouter", "stable").is_some());
}

#[tokio::test]
async fn openrouter_runtime_limit_rejects_overflow_atomically() {
    let _guard = test_mutation_lock().await;
    let mut models = (0..1_000)
        .map(|index| model(format!("vendor/model-{index}")))
        .collect::<Vec<_>>();
    replace_provider("openrouter", &models).unwrap();
    assert!(lookup("openrouter", "vendor/model-999").is_some());
    models.push(model("vendor/overflow".to_string()));
    assert_eq!(
        replace_provider("openrouter", &models),
        Err("too_many_models")
    );
    assert!(lookup("openrouter", "vendor/model-999").is_some());
    assert!(lookup("openrouter", "vendor/overflow").is_none());
}

#[tokio::test]
async fn catalogs_are_isolated_by_provider() {
    let _guard = test_mutation_lock().await;
    replace_provider("openrouter", &[model("shared".to_string())]).unwrap();
    replace_provider("openai", &[model("shared".to_string())]).unwrap();

    assert_eq!(
        lookup("openrouter", "shared").unwrap().max_output_tokens,
        Some(64_000)
    );
    assert!(lookup("unknown", "shared").is_none());
}

#[tokio::test]
async fn runtime_catalog_accepts_a_routed_model_suffix() {
    let _guard = test_mutation_lock().await;
    let id = "google/gemma-4-31b-it:free";

    replace_provider("openrouter", &[model(id.to_string())]).unwrap();

    assert!(lookup("openrouter", id).is_some());
}

#[test]
fn oldest_provider_is_evicted_at_capacity() {
    let mut registry = RuntimeRegistry::default();
    for index in 0..=MAX_RUNTIME_PROVIDERS {
        registry.replace(&format!("provider-{index}"), HashMap::new());
    }

    assert_eq!(registry.providers.len(), MAX_RUNTIME_PROVIDERS);
    assert!(!registry.providers.contains_key("provider-0"));
    assert!(registry
        .providers
        .contains_key(&format!("provider-{MAX_RUNTIME_PROVIDERS}")));
}
