use super::*;

#[tokio::test]
async fn beaver_registry_wins_over_litellm_for_direct_models() {
    assert_eq!(
        limits("xai", "grok-4.20-0309-reasoning").await,
        Some(ModelLimits {
            context_window: Some(1_000_000),
            max_output_tokens: None,
            default_output_tokens: None,
        })
    );
    assert_eq!(
        limits("openai", "o3").await,
        Some(ModelLimits {
            context_window: Some(200_000),
            max_output_tokens: Some(100_000),
            default_output_tokens: None,
        })
    );
}

#[tokio::test]
async fn an_unknown_local_output_does_not_reuse_litellms_copied_context() {
    assert_eq!(
        limits("mistral", "mistral-medium-3").await,
        Some(ModelLimits {
            context_window: Some(262_144),
            max_output_tokens: None,
            default_output_tokens: None,
        })
    );
    assert_eq!(
        limits("moonshot", "moonshot-v1-8k").await,
        Some(ModelLimits {
            context_window: Some(8_192),
            max_output_tokens: None,
            default_output_tokens: None,
        })
    );
}

#[tokio::test]
async fn aliases_keep_their_canonical_configuration() {
    assert_eq!(
        limits("xai", "grok-4.20").await,
        limits("xai", "grok-4.20-0309-reasoning").await
    );
    assert_eq!(
        local_capabilities("mistral", "mistral-small-latest"),
        local_capabilities("mistral", "mistral-small-2603")
    );
}

#[tokio::test]
async fn openrouter_inherits_beavers_upstream_model_configuration() {
    assert_eq!(
        limits("openrouter", "openai/o3").await,
        Some(ModelLimits {
            context_window: Some(200_000),
            max_output_tokens: Some(100_000),
            default_output_tokens: None,
        })
    );
    assert_eq!(
        local_capabilities("openrouter", "google/gemini-3.6-flash"),
        Some(ModelCapabilities {
            supports_tools: true,
            supports_vision: true,
            supports_thinking: true,
        })
    );
}

#[tokio::test]
async fn litellm_remains_the_fallback_for_unknown_local_models() {
    assert_eq!(
        limits("openai", "gpt-3.5-turbo").await,
        Some(ModelLimits {
            context_window: Some(16_385),
            max_output_tokens: Some(4_096),
            default_output_tokens: None,
        })
    );
    assert!(is_chat_model("openai", "gpt-3.5-turbo").await);
    assert_eq!(local_limits("openai", "foreign/o3"), None);
}

#[tokio::test]
async fn kimi_k3_keeps_its_documented_default_separate_from_its_maximum() {
    assert_eq!(
        limits("moonshot", "kimi-k3").await,
        Some(ModelLimits {
            context_window: Some(1_048_576),
            max_output_tokens: Some(1_048_576),
            default_output_tokens: Some(131_072),
        })
    );
}

#[test]
fn capability_resolution_reports_its_provenance() {
    let embedded = resolve_local("openai", "gpt-5.6-luna").unwrap();
    assert_eq!(embedded.provenance, CapabilityProvenance::EmbeddedRegistry);
    assert!(embedded.supports_tools);
    assert_eq!(embedded.reasoning_modes.len(), 6);

    let codex = resolve_local("codex-oauth", "gpt-5.6-luna").unwrap();
    assert_eq!(codex.provenance, CapabilityProvenance::ValidatedRuntime);
    assert!(codex.supports_tools);

    assert!(resolve_local("google", "gemini-unregistered-pro").is_none());
    assert!(resolve_local("unknown", "model").is_none());
}
