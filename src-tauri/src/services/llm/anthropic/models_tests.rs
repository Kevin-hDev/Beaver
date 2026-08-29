use serde_json::json;

const HAIKU: &str = "claude-haiku-4-5-20251001";

#[test]
fn catalog_intersects_the_single_validated_model_and_merges_missing_fields() {
    let response = json!({"data": [{
        "id": HAIKU,
        "display_name": "Claude Haiku 4.5",
        "max_input_tokens": 180000,
        "capabilities": {
            "tools": {"supported": false},
            "image_input": {"supported": true},
            "thinking": {"supported": true}
        }
    }, {"id": "claude-unvalidated"}]});

    let models = super::models::parse_and_intersect(&response).unwrap();

    assert_eq!(models.len(), 1);
    assert_eq!(models[0].id, HAIKU);
    assert_eq!(models[0].context_length, Some(180_000));
    assert!(!models[0].supports_tools);
    assert!(models[0].supports_vision);
    assert_eq!(models[0].max_output_tokens, Some(64_000));
}

#[test]
fn successful_catalog_without_haiku_stays_empty() {
    let models = super::models::parse_and_intersect(&json!({
        "data": [{"id": "claude-other"}]
    }))
    .unwrap();
    assert!(models.is_empty());
}

#[test]
fn catalog_rejects_oversize_and_invalid_ids() {
    let oversized = (0..501)
        .map(|index| json!({"id": format!("claude-{index}")}))
        .collect::<Vec<_>>();
    assert!(super::models::parse_and_intersect(&json!({"data": oversized})).is_err());
    assert!(super::models::parse_and_intersect(&json!({
        "data": [{"id": "../invalid"}]
    }))
    .is_err());
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
