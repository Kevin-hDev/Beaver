use super::types::ModelInfo;

#[tokio::test]
async fn live_google_resource_ids_reach_the_canonical_profile() {
    let _guard = super::runtime_models::test_mutation_lock().await;
    // Google /openai/models observed on 2026-09-07 returns resource IDs.
    let parsed = super::openai_compat_parsing::parse_models_list(
        &serde_json::json!({"data":[
            {"id":"models/gemini-3.8-flash", "owned_by":"google"},
            {"id":"gemini-3.8-flash", "owned_by":"google"}
        ]}),
        "google",
    )
    .unwrap();
    let models = super::model_catalog::enrich_models("google", parsed, false)
        .await
        .unwrap();
    assert_eq!(models.len(), 1);
    let model = &models[0];
    assert_eq!(model.id, "gemini-3.8-flash");
    assert!(model.supports_tools && model.supports_vision && model.supports_thinking);
    assert_eq!(model.reasoning_modes, ["low", "medium", "high"]);
    assert_eq!(model.default_reasoning_mode.as_deref(), Some("medium"));
    assert!(super::runtime_models::lookup("google", "gemini-3.8-flash").is_some());
    let other = super::openai_compat_parsing::parse_models_list(
        &serde_json::json!({"data":[{"id":"models/gemini-3.8-flash"}]}),
        "openrouter",
    )
    .unwrap();
    assert_eq!(other[0].id, "models/gemini-3.8-flash");
}

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
        reasoning_contract: None,
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
        reasoning_contract: None,
        supports_fast_mode: false,
        reasoning_modes: Vec::new(),
        default_reasoning_mode: None,
        context_usage_includes_reasoning: true,
        is_free: false,
    }
}

fn remote_openrouter_model(id: &str, reasoning_metadata_present: bool) -> ModelInfo {
    ModelInfo {
        id: id.into(),
        display_name: Some(id.into()),
        owned_by: Some("openrouter".into()),
        context_length: Some(1_310_720),
        max_output_tokens: Some(131_072),
        supports_tools: true,
        supports_vision: true,
        supports_thinking: true,
        reasoning_contract: reasoning_metadata_present.then_some({
            super::model_reasoning_contract::ModelReasoningContract {
                mandatory: None,
                default_enabled: None,
                supports_max_tokens: None,
                default_effort: None,
                control: super::model_reasoning_contract::ReasoningControl::ProviderDefault,
            }
        }),
        supports_fast_mode: false,
        reasoning_modes: Vec::new(),
        default_reasoning_mode: None,
        context_usage_includes_reasoning: true,
        is_free: false,
    }
}

fn remote_openrouter_catalog(count: usize) -> Vec<ModelInfo> {
    (0..count)
        .map(|index| remote_openrouter_model(&format!("vendor/model-{index}"), false))
        .collect()
}

#[tokio::test]
async fn native_catalog_keeps_explicit_remote_values() {
    let _guard = super::runtime_models::test_mutation_lock().await;
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
async fn catalog_rows_preserve_native_deduplication() {
    let _guard = super::runtime_models::test_mutation_lock().await;
    let model = remote_anthropic_model();
    let models = super::model_catalog::enrich_models("anthropic", vec![model.clone(), model], true)
        .await
        .unwrap();
    assert_eq!(models.len(), 1);
    assert!(super::runtime_models::lookup("anthropic", "claude-haiku-4-5-20251001").is_some());
}

#[tokio::test]
async fn duplicate_catalog_ids_are_rejected_before_runtime_registration() {
    let _guard = super::runtime_models::test_mutation_lock().await;
    super::runtime_models::replace_provider(
        "openrouter",
        &[remote_openrouter_model("stable-model", false)],
    )
    .unwrap();
    let model = remote_openrouter_model("vendor/duplicate", false);
    let result =
        super::model_catalog::enrich_models("openrouter", vec![model.clone(), model], false).await;

    assert!(matches!(result, Err(super::types::LlmError::Parse(_))));
    assert!(super::runtime_models::lookup("openrouter", "stable-model").is_some());
    assert!(super::runtime_models::lookup("openrouter", "vendor/duplicate").is_none());
}

#[tokio::test]
async fn catalog_rows_enrichment_retains_valid_ids_only() {
    let _guard = super::runtime_models::test_mutation_lock().await;
    let models = super::model_catalog::enrich_models(
        "openrouter",
        vec![
            remote_openrouter_model("../invalid", false),
            remote_openrouter_model("vendor/valid", false),
        ],
        false,
    )
    .await
    .unwrap();
    assert_eq!(models.len(), 1);
    assert_eq!(models[0].id, "vendor/valid");
    assert!(super::runtime_models::lookup("openrouter", "vendor/valid").is_some());
}

#[tokio::test]
async fn openrouter_keeps_its_full_bounded_catalog_in_any_order() {
    let _guard = super::runtime_models::test_mutation_lock().await;
    let forward = remote_openrouter_catalog(501);
    let mut reverse = forward.clone();
    reverse.reverse();

    let first = super::model_catalog::enrich_models("openrouter", forward, false)
        .await
        .unwrap();
    let second = super::model_catalog::enrich_models("openrouter", reverse, false)
        .await
        .unwrap();
    let ids = |models: &[ModelInfo]| {
        models
            .iter()
            .map(|model| model.id.clone())
            .collect::<std::collections::BTreeSet<_>>()
    };

    assert_eq!(first.len(), 501);
    assert_eq!(ids(&first), ids(&second));
    assert!(super::runtime_models::lookup("openrouter", "vendor/model-500").is_some());
}

#[tokio::test]
async fn oversized_openrouter_catalog_preserves_the_last_healthy_registry() {
    let _guard = super::runtime_models::test_mutation_lock().await;
    super::runtime_models::replace_provider(
        "openrouter",
        &[remote_openrouter_model("vendor/stable", false)],
    )
    .unwrap();

    let result =
        super::model_catalog::enrich_models("openrouter", remote_openrouter_catalog(1_001), false)
            .await;

    assert!(matches!(result, Err(super::types::LlmError::Parse(_))));
    assert!(super::runtime_models::lookup("openrouter", "vendor/stable").is_some());
    assert!(super::runtime_models::lookup("openrouter", "vendor/model-0").is_none());
}

#[tokio::test]
async fn qwen_remote_catalog_keeps_every_chat_model_returned_by_the_account() {
    let _guard = super::runtime_models::test_mutation_lock().await;
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
    let _guard = super::runtime_models::test_mutation_lock().await;
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
async fn qwen_hybrid_models_expose_their_real_thinking_switch() {
    let _guard = super::runtime_models::test_mutation_lock().await;
    let models =
        super::model_catalog::enrich_models("qwen", vec![remote_qwen_model("qwen3.7-plus")], false)
            .await
            .unwrap();

    assert_eq!(models.len(), 1);
    assert!(models[0].supports_thinking);
    assert_eq!(models[0].reasoning_modes, ["off", "auto"]);
    assert_eq!(models[0].default_reasoning_mode.as_deref(), Some("auto"));
}

#[tokio::test]
async fn anthropic_catalog_keeps_the_reasoning_modes_advertised_by_the_model() {
    let _guard = super::runtime_models::test_mutation_lock().await;
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
    let _guard = super::runtime_models::test_mutation_lock().await;
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

#[tokio::test]
async fn openrouter_explicit_empty_reasoning_is_not_reactivated_by_embedded_capabilities() {
    let _guard = super::runtime_models::test_mutation_lock().await;
    let models = super::model_catalog::enrich_models(
        "openrouter",
        vec![remote_openrouter_model("z-ai/glm-5.3-flash", true)],
        false,
    )
    .await
    .unwrap();

    assert_eq!(models.len(), 1);
    assert!(models[0].supports_thinking);
    // `auto` is the internal native-default state, not an exposed effort choice.
    assert_eq!(models[0].reasoning_modes, ["auto"]);
    assert_eq!(models[0].default_reasoning_mode.as_deref(), Some("auto"));
    assert_eq!(
        models[0].reasoning_contract.as_ref().unwrap().control,
        super::model_reasoning_contract::ReasoningControl::ProviderDefault
    );
    assert_eq!(models[0].context_length, Some(1_310_720));
    assert_eq!(models[0].max_output_tokens, Some(131_072));
}

#[tokio::test]
async fn openrouter_missing_reasoning_metadata_keeps_historical_capabilities() {
    let _guard = super::runtime_models::test_mutation_lock().await;
    let models = super::model_catalog::enrich_models(
        "openrouter",
        vec![remote_openrouter_model("z-ai/glm-5.3-flash", false)],
        false,
    )
    .await
    .unwrap();

    assert_eq!(models[0].reasoning_modes, ["low", "high", "max"]);
    assert_eq!(models[0].default_reasoning_mode.as_deref(), Some("max"));
}

#[tokio::test]
async fn invalid_openrouter_reasoning_never_reactivates_embedded_efforts() {
    let _guard = super::runtime_models::test_mutation_lock().await;
    let parsed = super::openai_compat_parsing::parse_models_list(
        &serde_json::json!({"data":[{
            "id":"z-ai/glm-5.3-flash",
            "supported_parameters":["reasoning"],
            "reasoning":{"supported_efforts":["quantum"]}
        }]}),
        "openrouter",
    )
    .unwrap();
    let models = super::model_catalog::enrich_models("openrouter", parsed, false)
        .await
        .unwrap();

    assert!(models.is_empty());
    assert!(super::runtime_models::lookup("openrouter", "z-ai/glm-5.3-flash").is_none());
}

#[tokio::test]
async fn explicit_remote_contract_and_catalog_projection_cannot_diverge() {
    let _guard = super::runtime_models::test_mutation_lock().await;
    let parsed = super::openai_compat_parsing::parse_models_list(
        &serde_json::json!({"data":[{
            "id":"z-ai/glm-5.3-flash", "supported_parameters":["reasoning"],
            "reasoning":{"supported_efforts":["low","high","max"]}
        }]}),
        "openrouter",
    )
    .unwrap();
    let models = super::model_catalog::enrich_models("openrouter", parsed, false)
        .await
        .unwrap();
    let model = &models[0];
    assert_eq!(
        model.default_reasoning_mode, None,
        "no embedded default over an explicit contract"
    );
    assert_eq!(
        model
            .reasoning_contract
            .as_ref()
            .unwrap()
            .legacy_projection(),
        (
            model.reasoning_modes.clone(),
            model.default_reasoning_mode.clone()
        )
    );
    super::runtime_models::replace_provider("openrouter", &[]).unwrap();
}

#[tokio::test]
async fn invalid_reasoning_shapes_never_become_absent_metadata() {
    let _guard = super::runtime_models::test_mutation_lock().await;
    for reasoning in [
        serde_json::json!(false),
        serde_json::json!(42),
        serde_json::json!("high"),
    ] {
        let parsed = super::openai_compat_parsing::parse_models_list(
            &serde_json::json!({"data":[{
                "id":"z-ai/glm-5.3-flash", "supported_parameters":["reasoning"], "reasoning":reasoning
            }]}), "openrouter",
        ).unwrap();
        let models = super::model_catalog::enrich_models("openrouter", parsed, false)
            .await
            .unwrap();
        assert!(models.is_empty());
    }
}

#[tokio::test]
async fn openrouter_catalog_enrichment_does_not_reuse_a_previous_runtime_restriction() {
    let _guard = super::runtime_models::test_mutation_lock().await;
    super::runtime_models::replace_provider(
        "openrouter",
        &[ModelInfo {
            reasoning_modes: vec!["low".into()],
            reasoning_contract: Some(super::model_reasoning_contract::ModelReasoningContract {
                mandatory: None,
                default_enabled: None,
                supports_max_tokens: None,
                default_effort: None,
                control: super::model_reasoning_contract::ReasoningControl::Efforts(vec![
                    crate::services::reasoning_continuity::contract::ReasoningModeId::Low,
                ]),
            }),
            ..remote_openrouter_model("z-ai/glm-5.3-flash", true)
        }],
    )
    .unwrap();

    let models = super::model_catalog::enrich_models(
        "openrouter",
        vec![remote_openrouter_model("z-ai/glm-5.3-flash", false)],
        false,
    )
    .await
    .unwrap();

    assert_eq!(models[0].reasoning_modes, ["low", "high", "max"]);
}

#[tokio::test]
async fn openrouter_catalog_default_and_limits_reach_the_backend_after_refresh() {
    let _guard = super::runtime_models::test_mutation_lock().await;
    for (efforts, default, expected) in [
        (
            serde_json::json!(["low", "high", "max"]),
            "low",
            Some("low"),
        ),
        (serde_json::json!(["medium"]), "medium", Some("medium")),
    ] {
        let parsed = super::openai_compat_parsing::parse_models_list(
            &serde_json::json!({"data":[{
                "id":"z-ai/glm-5.3-flash", "context_length":1310720,
                "top_provider":{"context_length":500000,"max_completion_tokens":4096},
                "supported_parameters":["tools","reasoning"],
                "reasoning":{"mandatory":true,"supported_efforts":efforts,"default_effort":default}
            }]}),
            "openrouter",
        )
        .unwrap();
        let models = super::model_catalog::enrich_models("openrouter", parsed, false)
            .await
            .unwrap();
        assert_eq!(models[0].default_reasoning_mode.as_deref(), expected);
        let backend =
            super::provider_model_lookup::resolve_local("openrouter", &models[0].id).unwrap();
        assert_eq!(backend.reasoning_modes, models[0].reasoning_modes);
        assert_eq!(
            backend.default_reasoning_mode,
            models[0].default_reasoning_mode
        );
        assert_eq!(
            crate::services::reasoning::normalize_for_model(
                "openrouter",
                &models[0].id,
                Some("off"),
                true
            )
            .as_deref(),
            expected
        );
        assert_eq!(
            super::model_context_length("openrouter", &models[0].id).await,
            Some(500000)
        );
        assert_eq!(
            super::stream_max_tokens::resolve(
                "openrouter",
                &models[0].id,
                Some(8000),
                true,
                None,
                1000
            )
            .await,
            Ok(Some(4096))
        );
    }
}
