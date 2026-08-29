use serde_json::json;

const HAIKU: &str = "claude-haiku-4-5-20251001";
const OPUS: &str = "claude-opus-4-8";

#[test]
fn catalog_keeps_every_available_claude_model_and_its_native_capabilities() {
    let response = json!({"data": [{
        "id": HAIKU,
        "display_name": "Claude Haiku 4.5",
        "max_input_tokens": 180000,
        "capabilities": {
            "tools": {"supported": false},
            "image_input": {"supported": true},
            "thinking": {"supported": true}
        }
    }, {
        "id": OPUS,
        "display_name": "Claude Opus 4.8",
        "max_input_tokens": 1000000,
        "max_tokens": 128000,
        "capabilities": {
            "image_input": {"supported": true},
            "thinking": {
                "supported": true,
                "types": {
                    "adaptive": {"supported": true},
                    "enabled": {"supported": false}
                }
            },
            "effort": {
                "supported": true,
                "low": {"supported": true},
                "medium": {"supported": true},
                "high": {"supported": true},
                "xhigh": {"supported": true},
                "max": {"supported": true}
            }
        }
    }]});

    let models = super::models::parse_catalog(&response).unwrap();

    assert_eq!(models.len(), 2);
    assert_eq!(models[0].id, HAIKU);
    assert_eq!(models[0].context_length, Some(180_000));
    assert!(!models[0].supports_tools);
    assert!(models[0].supports_vision);
    assert_eq!(models[0].max_output_tokens, Some(64_000));
    assert_eq!(models[1].id, OPUS);
    assert_eq!(models[1].context_length, Some(1_000_000));
    assert_eq!(models[1].max_output_tokens, Some(128_000));
    assert!(!models[1].supports_tools);
    assert!(models[1].supports_vision);
    assert!(models[1].supports_thinking);
    assert_eq!(
        models[1].reasoning_modes,
        ["off", "auto", "low", "medium", "high", "xhigh", "max"]
    );
}

#[test]
fn successful_catalog_without_haiku_keeps_the_available_model() {
    let models = super::models::parse_catalog(&json!({
        "data": [{"id": OPUS, "display_name": "Claude Opus 4.8"}]
    }))
    .unwrap();
    assert_eq!(models.len(), 1);
    assert_eq!(models[0].id, OPUS);
}

#[test]
fn unknown_zero_limits_do_not_reject_an_available_model() {
    let models = super::models::parse_catalog(&json!({
        "data": [{
            "id": OPUS,
            "display_name": "Claude Opus 4.8",
            "max_input_tokens": 0,
            "max_tokens": 0
        }]
    }))
    .unwrap();

    assert_eq!(models.len(), 1);
    assert_eq!(models[0].context_length, None);
    assert_eq!(models[0].max_output_tokens, None);
}

#[test]
fn catalog_rejects_oversize_and_skips_only_invalid_ids() {
    let oversized = (0..501)
        .map(|index| json!({"id": format!("claude-{index}")}))
        .collect::<Vec<_>>();
    assert!(super::models::parse_catalog(&json!({"data": oversized})).is_err());
    let models = super::models::parse_catalog(&json!({
        "data": [
            {"id": "../invalid"},
            {"id": OPUS, "display_name": "Claude Opus 4.8"}
        ]
    }))
    .unwrap();
    assert_eq!(models.len(), 1);
    assert_eq!(models[0].id, OPUS);
}

#[test]
fn missing_remote_capabilities_are_not_invented() {
    let models = super::models::parse_catalog(&json!({
        "data": [{"id": OPUS, "display_name": "Claude Opus 4.8"}]
    }))
    .unwrap();

    assert!(!models[0].supports_tools);
    assert!(!models[0].supports_vision);
}

#[test]
fn embedded_fallback_is_only_used_for_unavailable_endpoint() {
    let unavailable = Err(crate::services::llm::types::LlmError::Network(
        "offline".into(),
    ));
    let models = super::models::resolve_catalog(unavailable).unwrap();
    assert_eq!(models.len(), 1);
    assert_eq!(models[0].id, HAIKU);

    let rejected = Err(crate::services::llm::types::LlmError::Unauthorized);
    assert!(super::models::resolve_catalog(rejected).is_err());
}
