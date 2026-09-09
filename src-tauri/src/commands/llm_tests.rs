#[tokio::test]
async fn openrouter_model_keeps_upstream_tool_capability() {
    assert!(super::supports_tool_use("openrouter".to_string(), "openai/o3".to_string()).await);
}

#[test]
fn public_catalog_includes_live_anthropic_and_qwen_routes() {
    let public = super::list_llm_providers_catalog();
    let configurable = super::list_llm_configurable_providers_catalog();

    assert!(public.iter().any(|provider| provider.id == "anthropic"));
    assert_eq!(
        configurable
            .iter()
            .filter(|provider| provider.id == "anthropic")
            .count(),
        1
    );
    let qwen = configurable
        .iter()
        .filter(|provider| provider.id == "qwen")
        .collect::<Vec<_>>();
    assert_eq!(qwen.len(), 1);
    assert_eq!(
        qwen[0].connection_kind,
        crate::models::provider_contract::ProviderConnectionKind::QwenModelStudio
    );
    assert!(public.iter().any(|provider| provider.id == "qwen"));
    assert!(public.iter().all(|provider| configurable
        .iter()
        .any(|candidate| candidate.id == provider.id)));

    let mut ids = configurable
        .iter()
        .map(|provider| provider.id.as_str())
        .collect::<Vec<_>>();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), configurable.len());
}

#[tokio::test]
async fn model_context_comes_from_the_registered_route_metadata() {
    assert_eq!(
        crate::services::llm::model_context_length("openai", "gpt-5.6-sol").await,
        Some(1_050_000)
    );
    assert_eq!(
        crate::services::llm::model_context_length("openai", "unknown-model").await,
        None
    );
}

#[test]
fn api_catalog_commands_return_closed_error_codes() {
    use crate::services::llm::provider_error::ProviderErrorCode;
    use crate::services::llm::types::LlmError;

    for (error, expected) in [
        (LlmError::Unauthorized, "auth_failed"),
        (
            LlmError::KnownProvider(ProviderErrorCode::ProviderAccessUnavailable),
            "provider_access_unavailable",
        ),
        (
            LlmError::RateLimit {
                retry_after_secs: Some(3),
            },
            "rate_limit",
        ),
        (
            LlmError::Network("private".into()),
            "model_catalog_unavailable",
        ),
    ] {
        assert_eq!(super::api_catalog_error(error), expected);
    }
}
