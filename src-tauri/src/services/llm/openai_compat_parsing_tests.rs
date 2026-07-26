use super::openai_compat_parsing::{build_payload, parse_models_list};
use super::types::ChatRequest;
use serde_json::json;

#[test]
fn openrouter_models_use_supported_parameters_for_reasoning() {
    let body = json!({
        "data": [
            {
                "id": "provider/reasoning-model",
                "context_length": 128000,
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
    assert_eq!(
        reasoning.reasoning_modes,
        ["off", "auto", "low", "medium", "high", "xhigh"]
    );
    assert!(!plain.supports_thinking);
    assert!(plain.reasoning_modes.is_empty());
}

#[test]
fn google_models_use_name_based_reasoning_modes() {
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

    assert_eq!(gemini_35.reasoning_modes, ["low", "medium", "high"]);
    assert_eq!(gemini_25.reasoning_modes, ["off", "low", "medium", "high"]);
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
        assert_eq!(
            model.reasoning_modes,
            ["off", "low", "medium", "high", "xhigh", "max"]
        );
    }
}

#[test]
fn openrouter_new_models_use_provider_specific_reasoning_modes() {
    let body = json!({
        "data": [
            {
                "id": "openai/gpt-5.6-sol",
                "supported_parameters": ["tools", "reasoning"]
            },
            {
                "id": "openai/gpt-5.6-terra-pro",
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
    let terra_pro = models
        .iter()
        .find(|model| model.id == "openai/gpt-5.6-terra-pro")
        .unwrap();

    assert_eq!(
        sol.reasoning_modes,
        ["off", "low", "medium", "high", "xhigh", "max"]
    );
    assert_eq!(grok.reasoning_modes, ["low", "medium", "high"]);
    assert_eq!(terra_pro.reasoning_modes, sol.reasoning_modes);
}

#[test]
fn non_streaming_gpt_56_uses_max_completion_tokens() {
    let request = ChatRequest {
        model: "gpt-5.6-sol".to_string(),
        max_tokens: Some(4_096),
        ..ChatRequest::default()
    };

    let payload = build_payload(&request, false);

    assert_eq!(payload["max_completion_tokens"], 4_096);
    assert!(payload.get("max_tokens").is_none());
}
