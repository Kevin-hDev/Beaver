use super::xai_catalog_wire::{parse_catalog, XaiBackend};
use serde_json::json;

#[test]
fn parses_bounded_chat_and_responses_models() {
    let body = json!({"data": [
        {
            "model": "grok-4.6",
            "name": "Grok 4.6",
            "contextWindow": 500000,
            "apiBackend": "responses",
            "reasoningEfforts": ["low", {"value":"medium"}, {"value":"high"}, {"value":"xhigh"}],
            "reasoningEffort": "high"
        },
        {
            "model": "grok-4.5",
            "context_window": 500000,
            "api_backend": "chat_completions"
        }
    ]});

    let models = parse_catalog(&body).unwrap();
    assert_eq!(models.len(), 2);
    assert_eq!(models[0].backend, XaiBackend::Responses);
    assert_eq!(
        models[0].reasoning_modes,
        ["low", "medium", "high", "xhigh"]
    );
    assert_eq!(models[0].default_reasoning_mode.as_deref(), Some("high"));
    assert_eq!(models[1].backend, XaiBackend::ChatCompletions);
}

#[test]
fn excludes_models_that_are_not_visible_text_chat_models() {
    let body = json!({"data": [
        {
            "model": "grok-4.6",
            "contextWindow": 500000,
            "apiBackend": "responses",
            "inputModalities": ["text", "image"],
            "outputModalities": ["text"]
        },
        {
            "model": "grok-imagine-image",
            "apiBackend": "image_generations",
            "outputModalities": ["image"]
        },
        {
            "model": "hidden-chat-model",
            "contextWindow": 32000,
            "apiBackend": "chat_completions",
            "hidden": true
        },
        {
            "model": "unsupported-chat-model",
            "contextWindow": 32000,
            "apiBackend": "chat_completions",
            "supportedInApi": false
        }
    ]});

    let models = parse_catalog(&body).unwrap();

    assert_eq!(models.len(), 1);
    assert_eq!(models[0].id, "grok-4.6");
}

#[test]
fn rejects_duplicate_oversized_or_remote_routing_data() {
    let duplicate = json!({"data": [
        {"model":"grok-4.6","contextWindow":500000,"apiBackend":"responses"},
        {"model":"grok-4.6","contextWindow":500000,"apiBackend":"responses"}
    ]});
    assert_eq!(parse_catalog(&duplicate), Err("duplicate_model"));

    let oversized = json!({"data": (0..501).map(|index| json!({
        "model": format!("grok-{index}"), "contextWindow": 1000, "apiBackend": "responses"
    })).collect::<Vec<_>>()});
    assert_eq!(parse_catalog(&oversized), Err("model_count"));

    let routed = json!({"data": [{
        "model":"grok-4.6", "contextWindow":500000, "apiBackend":"responses",
        "baseUrl":"https://attacker.invalid/v1"
    }]});
    assert_eq!(parse_catalog(&routed), Err("remote_route"));
}

#[test]
fn filtered_models_ignore_unused_route_and_id_but_not_duplicate_ids() {
    let hidden_routed = json!({"data": [
        {"model":"grok-4.6", "contextWindow":500000, "apiBackend":"responses"},
        {
            "model":"grok-imagine", "apiBackend":"image_generations",
            "outputModalities":["image"], "baseUrl":"https://attacker.invalid/v1"
        }
    ]});
    assert_eq!(parse_catalog(&hidden_routed).unwrap().len(), 1);

    let hidden_invalid_id = json!({"data": [
        {"model":"grok-4.6", "contextWindow":500000, "apiBackend":"responses"},
        {"model":"../image", "apiBackend":"image_generations", "outputModalities":["image"]}
    ]});
    assert_eq!(parse_catalog(&hidden_invalid_id).unwrap().len(), 1);

    let hidden_duplicate = json!({"data": [
        {"model":"grok-4.6", "contextWindow":500000, "apiBackend":"responses"},
        {"model":"grok-imagine", "apiBackend":"image_generations", "outputModalities":["image"]},
        {"model":"grok-imagine", "apiBackend":"image_generations", "outputModalities":["image"]}
    ]});
    assert_eq!(parse_catalog(&hidden_duplicate), Err("duplicate_model"));
}

#[test]
fn rejects_unknown_reasoning_modes_and_backends() {
    let unknown_mode = json!({"data": [{
        "model":"grok-4.6", "contextWindow":500000, "apiBackend":"responses",
        "reasoningEfforts":["quantum"]
    }]});
    assert_eq!(parse_catalog(&unknown_mode), Err("reasoning_modes"));

    let unknown_backend = json!({"data": [{
        "model":"grok-4.6", "contextWindow":500000, "apiBackend":"caller_url"
    }]});
    assert_eq!(parse_catalog(&unknown_backend), Err("backend"));
}
