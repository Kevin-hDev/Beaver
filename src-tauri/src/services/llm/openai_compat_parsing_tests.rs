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
    assert!(reasoning.supports_thinking);
    assert!(reasoning.is_free);
    assert_eq!(reasoning.max_output_tokens, Some(65_535));
    assert!(reasoning.reasoning_modes.is_empty());
    assert!(!plain.supports_thinking);
    assert!(!plain.is_free);
    assert!(plain.reasoning_modes.is_empty());
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
fn invalid_provider_model_ids_and_metadata_are_filtered() {
    let body = json!({
        "data": [
            {"id": "../invalid"},
            {"id": "valid-model", "owned_by": "bad\nowner"}
        ]
    });

    let models = parse_models_list(&body, "openai").unwrap();

    assert_eq!(models.len(), 1);
    assert_eq!(models[0].id, "valid-model");
    assert!(models[0].owned_by.is_none());
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
