use super::{AuthSource, LlmRoute, UsageScope};

pub(super) fn test_route(chat_provider_id: &'static str) -> LlmRoute {
    // Ce chemin sans coffre permet aux tests HTTP de traverser l'envoi de production.
    LlmRoute {
        chat_provider_id,
        canonical_provider_id: chat_provider_id,
        base_url: "http://127.0.0.1".into(),
        models_endpoint: "/models".into(),
        display_name: "Test",
        auto_max_tokens: true,
        fallback_max_tokens: None,
        usage_scope: UsageScope::Any,
        error_policy: super::route_profile::ErrorPolicy::OpenAiCompatible,
        auth_source: AuthSource::TestToken("fixture-secret"),
    }
}
