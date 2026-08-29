use super::{
    capabilities_for, find_provider_entry, limits_for, CatalogCapabilities, CatalogLimits,
};
use crate::services::llm::litellm_catalog::parse_catalog;

fn embedded_registry() -> std::collections::HashMap<String, super::ModelEntry> {
    parse_catalog(include_str!("../../../resources/litellm-models.json"))
}

#[test]
fn direct_providers_use_their_own_model_limits() {
    let registry = embedded_registry();

    assert_eq!(
        limits_for(&registry, "openai", "o3"),
        Some(CatalogLimits {
            context_window: Some(200_000),
            max_output_tokens: Some(100_000),
        })
    );
    assert_eq!(
        limits_for(&registry, "openai", "gpt-4o"),
        Some(CatalogLimits {
            context_window: Some(128_000),
            max_output_tokens: Some(16_384),
        })
    );
}

#[test]
fn openrouter_uses_the_underlying_model_limit_not_its_stale_copy() {
    let registry = embedded_registry();

    assert_eq!(
        limits_for(&registry, "openrouter", "google/gemini-2.5-pro")
            .and_then(|limits| limits.max_output_tokens),
        Some(65_535)
    );
    assert_eq!(
        limits_for(&registry, "openrouter", "openai/gpt-4o")
            .and_then(|limits| limits.max_output_tokens),
        Some(16_384)
    );
    assert_eq!(
        limits_for(&registry, "openrouter", "openai/o3-mini")
            .and_then(|limits| limits.max_output_tokens),
        Some(100_000)
    );
}

#[test]
fn openrouter_recovers_upstream_capabilities_without_reusing_upstream_prices() {
    let registry = embedded_registry();

    assert!(find_provider_entry(&registry, "openrouter", "openai/o3").is_none());
    assert_eq!(
        capabilities_for(&registry, "openrouter", "openai/o3"),
        Some(CatalogCapabilities {
            supports_tools: true,
            supports_vision: true,
            supports_thinking: true,
        })
    );
}

#[test]
fn route_and_upstream_capabilities_are_merged() {
    let registry = parse_catalog(
        r#"{
            "openrouter/vendor/model":{"litellm_provider":"openrouter","mode":"chat","supports_function_calling":true},
            "model":{"litellm_provider":"vendor","mode":"chat","supports_vision":true,"supports_reasoning":true}
        }"#,
    );

    assert_eq!(
        capabilities_for(&registry, "openrouter", "vendor/model"),
        Some(CatalogCapabilities {
            supports_tools: true,
            supports_vision: true,
            supports_thinking: true,
        })
    );
}

#[test]
fn invalid_or_ambiguous_output_limits_fail_closed() {
    let registry = parse_catalog(
        r#"{
            "zero":{"litellm_provider":"openai","mode":"chat","max_input_tokens":100,"max_output_tokens":0},
            "oversized":{"litellm_provider":"openai","mode":"chat","max_input_tokens":100,"max_output_tokens":4294967296},
            "copied-context":{"litellm_provider":"openai","mode":"chat","max_input_tokens":100,"max_output_tokens":100}
        }"#,
    );

    assert_eq!(
        limits_for(&registry, "openai", "zero").and_then(|limits| limits.max_output_tokens),
        None
    );
    assert_eq!(
        limits_for(&registry, "openai", "oversized").and_then(|limits| limits.max_output_tokens),
        None
    );
    assert_eq!(
        limits_for(&registry, "openai", "copied-context")
            .and_then(|limits| limits.max_output_tokens),
        None
    );
}

#[test]
fn a_foreign_owner_cannot_reuse_another_direct_providers_model() {
    let registry = embedded_registry();

    assert!(find_provider_entry(&registry, "openai", "foreign/o3").is_none());
    assert!(find_provider_entry(&registry, "openai", "openai/o3").is_some());
}

#[test]
fn qwen_uses_the_dashscope_catalog_namespace() {
    let registry = parse_catalog(
        r#"{
            "dashscope/qwen3.8-max":{
                "litellm_provider":"dashscope",
                "mode":"chat",
                "supports_function_calling":true,
                "supports_vision":true,
                "supports_reasoning":true,
                "max_input_tokens":1000000,
                "max_output_tokens":131072
            }
        }"#,
    );

    assert_eq!(
        capabilities_for(&registry, "qwen", "qwen3.8-max"),
        Some(CatalogCapabilities {
            supports_tools: true,
            supports_vision: true,
            supports_thinking: true,
        })
    );
    assert_eq!(
        limits_for(&registry, "qwen", "qwen3.8-max"),
        Some(CatalogLimits {
            context_window: Some(1_000_000),
            max_output_tokens: Some(131_072),
        })
    );
}
