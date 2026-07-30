use super::{capabilities_for, find_provider_entry, max_output_tokens_for, ModelCapabilities};
use crate::services::llm::model_registry::parse_registry;

fn embedded_registry() -> std::collections::HashMap<String, super::ModelEntry> {
    parse_registry(include_str!("../../../resources/litellm-models.json"))
}

#[test]
fn direct_providers_use_their_own_model_limits() {
    let registry = embedded_registry();

    assert_eq!(
        max_output_tokens_for(&registry, "openai", "o3"),
        Some(100_000)
    );
    assert_eq!(
        max_output_tokens_for(&registry, "openai", "gpt-4o"),
        Some(16_384)
    );
}

#[test]
fn openrouter_uses_the_underlying_model_limit_not_its_stale_copy() {
    let registry = embedded_registry();

    assert_eq!(
        max_output_tokens_for(&registry, "openrouter", "google/gemini-2.5-pro"),
        Some(65_535)
    );
    assert_eq!(
        max_output_tokens_for(&registry, "openrouter", "openai/gpt-4o"),
        Some(16_384)
    );
    assert_eq!(
        max_output_tokens_for(&registry, "openrouter", "openai/o3-mini"),
        Some(100_000)
    );
}

#[test]
fn openrouter_recovers_upstream_capabilities_without_reusing_upstream_prices() {
    let registry = embedded_registry();

    assert!(find_provider_entry(&registry, "openrouter", "openai/o3").is_none());
    assert_eq!(
        capabilities_for(&registry, "openrouter", "openai/o3"),
        Some(ModelCapabilities {
            supports_tools: true,
            supports_vision: true,
            supports_thinking: true,
        })
    );
}

#[test]
fn route_and_upstream_capabilities_are_merged() {
    let registry = parse_registry(
        r#"{
            "openrouter/vendor/model":{"litellm_provider":"openrouter","mode":"chat","supports_function_calling":true},
            "model":{"litellm_provider":"vendor","mode":"chat","supports_vision":true,"supports_reasoning":true}
        }"#,
    );

    assert_eq!(
        capabilities_for(&registry, "openrouter", "vendor/model"),
        Some(ModelCapabilities {
            supports_tools: true,
            supports_vision: true,
            supports_thinking: true,
        })
    );
}

#[test]
fn invalid_token_limits_fail_closed() {
    let registry = parse_registry(
        r#"{
            "zero":{"litellm_provider":"openai","mode":"chat","max_output_tokens":0},
            "oversized":{"litellm_provider":"openai","mode":"chat","max_output_tokens":4294967296}
        }"#,
    );

    assert_eq!(max_output_tokens_for(&registry, "openai", "zero"), None);
    assert_eq!(
        max_output_tokens_for(&registry, "openai", "oversized"),
        None
    );
}
