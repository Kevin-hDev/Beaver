use super::provider_model_capabilities::{resolve_local, CapabilityProvenance};
use super::types::ModelInfo;

#[test]
fn codex_runtime_catalog_resolves_a_model_absent_from_the_fallback() {
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
