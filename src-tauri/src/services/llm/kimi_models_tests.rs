use super::kimi_models::parse_models_list;
use serde_json::json;

#[test]
fn parses_official_k3_and_k27_metadata() {
    let models = parse_models_list(&json!({
        "data": [
            {
                "id": "k3",
                "display_name": "K3",
                "context_length": 1_048_576,
                "supports_reasoning": true,
                "supports_image_in": true,
                "supports_tool_use": true,
                "supports_thinking_type": "only",
                "think_efforts": {
                    "support": true,
                    "valid_efforts": ["low", "high", "max"],
                    "default_effort": "max"
                }
            },
            {
                "id": "kimi-for-coding",
                "display_name": "K2.7 Coding",
                "context_length": 262_144,
                "supports_reasoning": true,
                "supports_image_in": true,
                "supports_tool_use": true,
                "supports_thinking_type": "only"
            }
        ]
    }))
    .unwrap();

    let k3 = &models[0];
    assert_eq!(k3.display_name.as_deref(), Some("K3"));
    assert_eq!(k3.context_length, Some(1_048_576));
    assert!(k3.supports_tools && k3.supports_vision && k3.supports_thinking);
    assert_eq!(k3.reasoning_modes, ["low", "high", "max"]);
    assert_eq!(k3.default_reasoning_mode.as_deref(), Some("max"));

    let k27 = &models[1];
    assert_eq!(k27.display_name.as_deref(), Some("K2.7 Coding"));
    assert_eq!(k27.reasoning_modes, ["auto"]);
    assert_eq!(k27.default_reasoning_mode.as_deref(), Some("auto"));
}

#[test]
fn uses_known_kimi_fallbacks_without_guessing_unknown_models() {
    let models = parse_models_list(&json!({
        "data": [
            { "id": "k3", "context_length": 262_144 },
            { "id": "future-model", "context_length": 128_000 }
        ]
    }))
    .unwrap();

    assert_eq!(models[0].display_name.as_deref(), Some("K3"));
    assert!(models[0].supports_tools);
    assert!(models[0].supports_vision);
    assert_eq!(models[0].reasoning_modes, ["low", "high", "max"]);
    assert!(!models[1].supports_tools);
    assert!(!models[1].supports_vision);
    assert!(!models[1].supports_thinking);
}

#[test]
fn bounds_and_validates_provider_metadata() {
    let long_name = "x".repeat(97);
    let models = parse_models_list(&json!({
        "data": [{
            "id": "k3",
            "display_name": long_name,
            "context_length": 262_144,
            "supports_tool_use": "yes",
            "supports_thinking_type": "only",
            "think_efforts": {
                "support": true,
                "valid_efforts": ["low", "low", "invalid", "high", "max"],
                "default_effort": "invalid"
            }
        }]
    }))
    .unwrap();

    assert_eq!(models[0].display_name.as_deref(), Some("K3"));
    assert!(!models[0].supports_tools);
    assert_eq!(models[0].reasoning_modes, ["low", "high", "max"]);
    assert_eq!(models[0].default_reasoning_mode.as_deref(), Some("max"));
    assert!(parse_models_list(&json!({ "data": [{ "id": "k3" }] })).is_err());
    assert!(parse_models_list(&json!({ "models": [] })).is_err());
}
