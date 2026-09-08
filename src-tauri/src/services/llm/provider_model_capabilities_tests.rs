use super::provider_model_capabilities::{resolve_local, CapabilityProvenance};
use super::types::ModelInfo;

#[tokio::test]
async fn codex_runtime_catalog_resolves_a_model_absent_from_the_fallback() {
    let _guard = super::runtime_models::test_mutation_lock().await;
    let model_id = "gpt-runtime-only-fixture";
    super::runtime_models::replace_provider(
        crate::services::codex_client::PROVIDER_ID,
        &[ModelInfo {
            id: model_id.to_string(),
            display_name: Some("Runtime fixture".to_string()),
            owned_by: Some("openai".to_string()),
            context_length: Some(128_000),
            max_output_tokens: Some(32_000),
            supports_tools: true,
            supports_vision: true,
            supports_thinking: true,
            reasoning_metadata_present: false,
            supports_fast_mode: false,
            reasoning_modes: vec!["low".to_string(), "high".to_string()],
            default_reasoning_mode: Some("high".to_string()),
            context_usage_includes_reasoning: false,
            is_free: false,
        }],
    );

    let resolved = resolve_local(crate::services::codex_client::PROVIDER_ID, model_id)
        .expect("a validated runtime Codex model must keep its capabilities");

    assert_eq!(resolved.provenance, CapabilityProvenance::ValidatedRuntime);
    assert!(resolved.supports_tools);
    assert!(resolved.supports_vision);
    assert!(resolved.supports_thinking);
    assert_eq!(resolved.reasoning_modes, ["low", "high"]);
}

#[tokio::test]
async fn openrouter_explicit_empty_reasoning_stays_empty_in_backend_normalization() {
    let _guard = super::runtime_models::test_mutation_lock().await;
    super::runtime_models::replace_provider(
        "openrouter",
        &[ModelInfo {
            id: "openai/o3".to_string(),
            display_name: None,
            owned_by: Some("openrouter".to_string()),
            context_length: Some(200_000),
            max_output_tokens: Some(100_000),
            supports_tools: true,
            supports_vision: true,
            supports_thinking: true,
            reasoning_metadata_present: true,
            supports_fast_mode: false,
            reasoning_modes: Vec::new(),
            default_reasoning_mode: None,
            context_usage_includes_reasoning: true,
            is_free: false,
        }],
    );

    assert!(
        super::provider_model_lookup::resolve_reasoning_modes("openrouter", "openai/o3", true)
            .is_empty()
    );
    assert_eq!(
        crate::services::reasoning::normalize_for_model(
            "openrouter",
            "openai/o3",
            Some("medium"),
            true,
        ),
        None
    );
}
