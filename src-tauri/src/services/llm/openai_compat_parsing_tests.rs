use super::openai_compat_parsing::parse_models_list;
use serde_json::json;

#[test]
fn openrouter_models_use_supported_parameters_for_reasoning() {
    let body = json!({
        "data": [
            {
                "id": "provider/reasoning-model",
                "context_length": 128000,
                "top_provider": { "max_completion_tokens": 65535 },
                "pricing": { "prompt": "0", "completion": "0" },
                "supported_parameters": ["tools", "reasoning", "include_reasoning"]
            },
            {
                "id": "provider/plain-model",
                "pricing": { "prompt": "0.1", "completion": "0.2" },
                "supported_parameters": ["tools"]
            }
        ]
    });

    let models = parse_models_list(&body, "openrouter").unwrap();
    let reasoning = models
        .iter()
        .find(|m| m.id == "provider/reasoning-model")
        .unwrap();
    let plain = models
        .iter()
        .find(|m| m.id == "provider/plain-model")
        .unwrap();

    assert!(reasoning.supports_tools);
    assert_eq!(
        reasoning
            .supported_parameters
            .as_ref()
            .unwrap()
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["tools", "reasoning", "include_reasoning"]
    );
    assert!(reasoning.supports_thinking);
    assert!(reasoning.is_free);
    assert_eq!(reasoning.max_output_tokens, Some(65_535));
    assert!(reasoning.reasoning_modes.is_empty());
    assert!(!plain.supports_thinking);
    assert_eq!(plain.supported_parameters.as_deref().unwrap(), ["tools"]);
    assert!(!plain.is_free);
    assert!(plain.reasoning_modes.is_empty());
}

#[test]
fn openrouter_supported_parameters_preserve_absence_and_reject_invalid_lists() {
    let too_many = (0..65)
        .map(|index| format!("parameter-{index}"))
        .collect::<Vec<_>>();
    let body = json!({"data": [
        {"id":"vendor/published", "supported_parameters":["tools"]},
        {"id":"vendor/absent"},
        {"id":"vendor/non-string", "supported_parameters":["tools", 3]},
        {"id":"vendor/too-many", "supported_parameters":too_many},
        {"id":"vendor/too-long", "supported_parameters":["x".repeat(65)]}
    ]});

    let models = parse_models_list(&body, "openrouter").unwrap();

    assert_eq!(models.len(), 2);
    assert_eq!(
        models[0].supported_parameters.as_deref().unwrap(),
        ["tools"]
    );
    assert!(models[1].supported_parameters.is_none());
}

#[test]
fn openrouter_catalog_keeps_only_models_that_publish_text_output() {
    let body = json!({"data": [
        {"id":"vendor/text", "architecture":{"output_modalities":["text"]}},
        {"id":"vendor/image", "architecture":{"output_modalities":["image"]}},
        {"id":"vendor/text-image", "architecture":{"output_modalities":["text","image"]}}
    ]});

    let models = parse_models_list(&body, "openrouter").unwrap();

    assert_eq!(
        models
            .iter()
            .map(|model| model.id.as_str())
            .collect::<Vec<_>>(),
        ["vendor/text", "vendor/text-image"]
    );
}

#[tokio::test]
async fn openrouter_explicit_capability_absence_is_not_overwritten_by_embedded_data() {
    let _guard = super::runtime_models::test_mutation_lock().await;
    let explicit = parse_models_list(
        &json!({"data": [{
            "id":"z-ai/glm-5.3-flash",
            "architecture":{"input_modalities":["text"],"output_modalities":["text"]},
            "supported_parameters":[]
        }]}),
        "openrouter",
    )
    .unwrap()
    .remove(0);
    let explicit = super::model_catalog::enrich_models("openrouter", vec![explicit], false)
        .await
        .unwrap()
        .remove(0);
    let absent = parse_models_list(
        &json!({"data": [{"id":"z-ai/glm-5.3-flash"}]}),
        "openrouter",
    )
    .unwrap()
    .remove(0);
    let absent = super::model_catalog::enrich_models("openrouter", vec![absent], false)
        .await
        .unwrap()
        .remove(0);

    assert!(!explicit.supports_tools);
    assert!(!explicit.supports_vision);
    assert!(!explicit.supports_thinking);
    assert!(absent.supports_tools);
    assert!(absent.supports_vision);
    assert!(absent.supports_thinking);
}

#[test]
fn feature_flags_do_not_invent_dynamic_reasoning_levels() {
    let body = json!({
        "data": [{
            "id": "provider/reasoning-model",
            "supported_parameters": ["reasoning_effort"]
        }]
    });

    let model = parse_models_list(&body, "openrouter").unwrap().remove(0);

    assert!(model.supports_thinking);
    assert!(model.reasoning_modes.is_empty());
    assert!(model.default_reasoning_mode.is_none());
}

#[tokio::test]
async fn root_catalog_regression_capabilities_keep_independent_presence() {
    let _guard = super::runtime_models::test_mutation_lock().await;
    let rows = [
        json!({"id":"z-ai/glm-5.3-flash", "architecture":{"input_modalities":["text"]}}),
        json!({"id":"z-ai/glm-5.3-flash", "supported_parameters":["tools"]}),
        json!({"id":"z-ai/glm-5.3-flash", "capabilities":{"function_calling":false}}),
        json!({"id":"vendor/fixed", "reasoning":{"mandatory":true}}),
    ];
    let mut actual = Vec::new();
    for row in rows {
        let parsed = parse_models_list(&json!({"data":[row]}), "openrouter").unwrap();
        let models = super::model_catalog::enrich_models("openrouter", parsed, false)
            .await
            .unwrap();
        let model = &models[0];
        actual.push((
            model.supports_tools,
            model.supports_vision,
            model.supports_thinking,
        ));
    }
    assert_eq!(
        actual,
        [
            (true, false, true),
            (true, true, false),
            (false, true, true),
            (false, false, true)
        ]
    );
}

#[test]
fn disabled_dynamic_reasoning_never_publishes_a_static_default() {
    let body = json!({"data": [{"id": "x-ai/grok-4.6"}]});

    let model = parse_models_list(&body, "openrouter").unwrap().remove(0);

    assert!(!model.supports_thinking);
    assert!(model.reasoning_modes.is_empty());
    assert!(model.default_reasoning_mode.is_none());
}

#[test]
fn unpriced_or_partially_priced_models_are_not_marked_free() {
    let body = json!({
        "data": [
            {"id": "provider/unpriced"},
            {"id": "provider/image-cost", "pricing": {
                "prompt": "0", "completion": "0", "image": "0.01"
            }}
        ]
    });

    let models = parse_models_list(&body, "openai").unwrap();
    assert!(models.iter().all(|model| !model.is_free));
}

#[test]
fn invalid_runtime_output_limits_are_ignored() {
    let body = json!({
        "data": [
            {
                "id": "provider/zero",
                "top_provider": { "max_completion_tokens": 0 }
            },
            {
                "id": "provider/oversized",
                "top_provider": { "max_completion_tokens": 4294967296_u64 }
            }
        ]
    });

    let models = parse_models_list(&body, "openrouter").unwrap();

    assert!(models.iter().all(|model| model.max_output_tokens.is_none()));
}

#[test]
fn catalog_rows_keep_valid_models_when_one_id_is_invalid() {
    let body = json!({
        "data": [
            {"id": "../invalid"},
            {"id": "valid-model", "owned_by": "bad\nowner"}
        ]
    });

    for provider in ["openai", "openrouter"] {
        let models = parse_models_list(&body, provider).unwrap();
        assert_eq!(models.len(), 1);
        assert_eq!(models[0].id, "valid-model");
        assert!(models[0].owned_by.is_none());
    }
}

#[test]
fn openrouter_parser_keeps_its_exact_limit_and_rejects_one_more() {
    let body = |count| {
        json!({
            "data": (0..count)
                .map(|index| json!({"id": format!("vendor/model-{index}")}))
                .collect::<Vec<_>>()
        })
    };

    assert_eq!(
        parse_models_list(&body(1_000), "openrouter").unwrap().len(),
        1_000
    );
    assert!(matches!(
        parse_models_list(&body(1_001), "openrouter"),
        Err(super::types::LlmError::Parse(_))
    ));
}

#[test]
fn openrouter_parser_rejects_an_exact_duplicate_id() {
    let body = json!({"data": [
        {"id": "vendor/model"},
        {"id": "vendor/model"}
    ]});

    assert!(matches!(
        parse_models_list(&body, "openrouter"),
        Err(super::types::LlmError::Parse(_))
    ));
}

#[test]
fn local_limits_override_conflicting_runtime_metadata() {
    let body = json!({
        "data": [{
            "id": "o3",
            "context_length": 8_192,
            "max_output_tokens": 4_096
        }]
    });

    let model = parse_models_list(&body, "openai").unwrap().remove(0);

    assert_eq!(model.context_length, Some(200_000));
    assert_eq!(model.max_output_tokens, Some(100_000));
}

#[test]
fn generic_google_catalog_does_not_invent_reasoning_modes() {
    let body = json!({
        "data": [
            {
                "id": "gemini-3.5-flash",
                "capabilities": { "completion_chat": true },
                "pricing": { "prompt": "0", "completion": "0" }
            },
            {
                "id": "gemini-2.5-flash",
                "capabilities": { "completion_chat": true },
                "pricing": { "prompt": "0", "completion": "0" }
            }
        ]
    });

    let models = parse_models_list(&body, "google").unwrap();
    let gemini_35 = models.iter().find(|m| m.id == "gemini-3.5-flash").unwrap();
    let gemini_25 = models.iter().find(|m| m.id == "gemini-2.5-flash").unwrap();

    assert!(gemini_35.reasoning_modes.is_empty());
    assert!(gemini_25.reasoning_modes.is_empty());
}

#[test]
fn openai_gpt_56_models_receive_official_capabilities() {
    let body = json!({
        "data": [
            { "id": "gpt-5.6-sol", "owned_by": "openai" },
            { "id": "gpt-5.6-terra", "owned_by": "openai" },
            { "id": "gpt-5.6-luna", "owned_by": "openai" },
            { "id": "gpt-5.6", "owned_by": "openai" }
        ]
    });

    let models = parse_models_list(&body, "openai").unwrap();

    assert_eq!(models.len(), 4);
    for model in models {
        assert_eq!(model.context_length, Some(1_050_000));
        assert!(model.supports_tools);
        assert!(model.supports_vision);
        assert!(model.supports_thinking);
        assert!(model.reasoning_modes.is_empty());
    }
}

#[test]
fn openrouter_feature_flags_do_not_duplicate_static_reasoning_modes() {
    let body = json!({
        "data": [
            {
                "id": "openai/gpt-5.6-sol",
                "supported_parameters": ["tools", "reasoning"]
            },
            {
                "id": "openai/gpt-5.6-terra",
                "supported_parameters": ["tools", "reasoning"]
            },
            {
                "id": "x-ai/grok-4.5",
                "supported_parameters": ["tools", "reasoning"]
            }
        ]
    });

    let models = parse_models_list(&body, "openrouter").unwrap();
    let sol = models
        .iter()
        .find(|model| model.id == "openai/gpt-5.6-sol")
        .unwrap();
    let grok = models
        .iter()
        .find(|model| model.id == "x-ai/grok-4.5")
        .unwrap();
    let terra = models
        .iter()
        .find(|model| model.id == "openai/gpt-5.6-terra")
        .unwrap();

    assert!(sol.reasoning_modes.is_empty());
    assert!(grok.reasoning_modes.is_empty());
    assert!(terra.reasoning_modes.is_empty());
}

#[test]
fn openrouter_reasoning_metadata_and_effective_limits_are_preserved() {
    let body = json!({
        "data": [{
            "id": "z-ai/glm-5.3-flash",
            "context_length": 1_310_720,
            "top_provider": {
                "context_length": 1_048_576,
                "max_completion_tokens": 131072
            },
            "reasoning": {
                "mandatory": true,
                "supported_efforts": ["max", "high", "low"],
                "default_effort": "max"
            },
            "supported_parameters": ["reasoning", "tools"]
        }]
    });

    let model = parse_models_list(&body, "openrouter").unwrap().remove(0);

    assert_eq!(model.context_length, Some(1_048_576));
    assert_eq!(model.max_output_tokens, Some(131072));
    assert!(model.supports_thinking);
    assert!(model.reasoning_contract.is_some());
    assert_eq!(model.reasoning_modes, ["max", "high", "low"]);
    assert_eq!(model.default_reasoning_mode.as_deref(), Some("max"));
}

#[test]
fn openrouter_empty_efforts_use_the_provider_default_without_inventing_a_level() {
    let body = json!({
        "data": [{
            "id": "openai/o3",
            "supported_parameters": ["reasoning"],
            "reasoning": {"supported_efforts": []}
        }]
    });

    let model = parse_models_list(&body, "openrouter").unwrap().remove(0);

    assert!(model.supports_thinking);
    assert_eq!(model.reasoning_modes, ["auto"]);
    assert_eq!(model.default_reasoning_mode.as_deref(), Some("auto"));
    assert!(model.reasoning_contract.is_some());
}

#[test]
fn openrouter_unknown_effort_keeps_the_model_on_provider_default() {
    let body = json!({
        "data": [{
            "id": "future/model",
            "supported_parameters": ["reasoning", "tools"],
            "reasoning": {
                "mandatory": true,
                "supported_efforts": ["quantum"]
            }
        }]
    });

    let models = parse_models_list(&body, "openrouter").unwrap();
    let model = models
        .first()
        .expect("the usable model must remain visible");
    assert!(model.supports_thinking);
    assert_eq!(model.reasoning_modes, ["auto"]);
    assert_eq!(model.default_reasoning_mode.as_deref(), Some("auto"));
    assert!(matches!(
        model
            .reasoning_contract
            .as_ref()
            .map(|contract| &contract.control),
        Some(super::model_reasoning_contract::ReasoningControl::ProviderDefault)
    ));
}

#[test]
fn malformed_supported_parameters_only_reject_openrouter_catalog_entries() {
    let body = json!({
        "data": [{
            "id": "mistral-test",
            "supported_parameters": "unexpected"
        }]
    });

    let mistral = parse_models_list(&body, "mistral").unwrap();
    assert_eq!(mistral.len(), 1);
    assert!(mistral[0].supported_parameters.is_none());

    let openrouter = parse_models_list(&body, "openrouter").unwrap();
    assert!(openrouter.is_empty());
}

#[test]
fn openrouter_documented_optional_reasoning_projects_a_toggle_without_efforts() {
    let body = json!({
        "data": [{
            "id": "qwen/qwen3.8-flash",
            "reasoning": {
                "mandatory": false,
                "default_enabled": true,
                "supports_max_tokens": true
            },
            "supported_parameters": ["reasoning"]
        }]
    });

    let model = parse_models_list(&body, "openrouter").unwrap().remove(0);

    assert!(model.supports_thinking);
    assert_eq!(model.reasoning_modes, ["off", "auto"]);
    assert_eq!(model.default_reasoning_mode.as_deref(), Some("auto"));
    assert!(model.reasoning_contract.is_some());
}

#[test]
fn openrouter_output_limit_uses_the_most_restrictive_positive_remote_value() {
    let body = json!({
        "data": [{
            "id": "openai/gpt-6-astra",
            "top_provider": {"max_completion_tokens": 131072},
            "max_output_tokens": 65536
        }]
    });

    let model = parse_models_list(&body, "openrouter").unwrap().remove(0);

    assert_eq!(model.max_output_tokens, Some(65536));
}
