use super::codex;
use crate::services::llm::{runtime_models, types::ModelInfo};

#[tokio::test]
async fn codex_catalog_default_is_validated_or_replaced_by_a_published_mode() {
    let _guard = runtime_models::test_mutation_lock().await;
    let mut model = ModelInfo {
        id: "gpt-runtime-default-max".into(),
        display_name: Some("Runtime fixture".into()),
        owned_by: Some("openai".into()),
        context_length: Some(128_000),
        max_output_tokens: None,
        supports_tools: true,
        supports_vision: true,
        supports_thinking: true,
        reasoning_contract: None,
        supports_fast_mode: false,
        reasoning_modes: vec!["low".into(), "max".into()],
        default_reasoning_mode: Some("max".into()),
        context_usage_includes_reasoning: true,
        is_free: false,
    };
    runtime_models::replace_provider("codex-oauth", &[model.clone()]).unwrap();
    assert_eq!(codex(&model.id, None), "max");
    assert_eq!(codex(&model.id, Some("low")), "low");

    model.default_reasoning_mode = None;
    runtime_models::replace_provider("codex-oauth", &[model.clone()]).unwrap();
    assert!(runtime_models::lookup("codex-oauth", &model.id).is_some());
    for requested in [None, Some("off"), Some("medium")] {
        assert_eq!(codex(&model.id, requested), "low");
    }
    // Invalid defaults cannot enter the runtime registry at all.
    model.default_reasoning_mode = Some("high".into());
    runtime_models::replace_provider("codex-oauth", &[model.clone()]).unwrap();
    assert!(runtime_models::lookup("codex-oauth", &model.id).is_none());
    runtime_models::replace_provider("codex-oauth", &[]).unwrap();
}
