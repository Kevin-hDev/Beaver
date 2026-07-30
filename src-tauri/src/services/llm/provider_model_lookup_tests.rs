use super::*;

#[tokio::test]
async fn beaver_registry_wins_over_litellm_for_direct_models() {
    assert_eq!(
        limits("xai", "grok-4.20-0309-reasoning").await,
        Some(ModelLimits {
            context_window: Some(1_000_000),
            max_output_tokens: None,
        })
    );
    assert_eq!(
        limits("openai", "o3").await,
        Some(ModelLimits {
            context_window: Some(200_000),
            max_output_tokens: Some(100_000),
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
        })
    );
    assert_eq!(
        limits("moonshot", "moonshot-v1-8k").await,
        Some(ModelLimits {
            context_window: Some(8_192),
            max_output_tokens: None,
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
        capabilities("mistral", "mistral-small-latest").await,
        capabilities("mistral", "mistral-small-2603").await
    );
}

#[tokio::test]
async fn openrouter_inherits_beavers_upstream_model_configuration() {
    assert_eq!(
        limits("openrouter", "openai/o3").await,
        Some(ModelLimits {
            context_window: Some(200_000),
            max_output_tokens: Some(100_000),
        })
    );
    assert_eq!(
        capabilities("openrouter", "google/gemini-3.6-flash").await,
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
        })
    );
    assert!(is_chat_model("openai", "gpt-3.5-turbo").await);
    assert_eq!(local_limits("openai", "foreign/o3"), None);
}
