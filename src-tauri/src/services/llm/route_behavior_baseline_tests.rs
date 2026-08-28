use super::route::{self, UsageScope};

struct ExpectedRoute {
    id: &'static str,
    canonical: &'static str,
    base_url: &'static str,
    models_endpoint: &'static str,
    oauth: bool,
    auto_max_tokens: bool,
    fallback_max_tokens: Option<u32>,
    usage_scope: UsageScope,
}

#[test]
fn route_behavior_baseline_preserves_every_remote_route() {
    let expected = [
        ExpectedRoute {
            id: "google",
            canonical: "google",
            base_url: "https://generativelanguage.googleapis.com/v1beta/openai",
            models_endpoint: "/models",
            oauth: false,
            auto_max_tokens: true,
            fallback_max_tokens: None,
            usage_scope: UsageScope::Any,
        },
        ExpectedRoute {
            id: "mistral",
            canonical: "mistral",
            base_url: "https://api.mistral.ai/v1",
            models_endpoint: "/models",
            oauth: false,
            auto_max_tokens: true,
            fallback_max_tokens: Some(64_000),
            usage_scope: UsageScope::Any,
        },
        ExpectedRoute {
            id: "cerebras",
            canonical: "cerebras",
            base_url: "https://api.cerebras.ai/v1",
            models_endpoint: "/models",
            oauth: false,
            auto_max_tokens: false,
            fallback_max_tokens: None,
            usage_scope: UsageScope::Any,
        },
        ExpectedRoute {
            id: "openrouter",
            canonical: "openrouter",
            base_url: "https://openrouter.ai/api/v1",
            models_endpoint: "/models",
            oauth: false,
            auto_max_tokens: true,
            fallback_max_tokens: Some(64_000),
            usage_scope: UsageScope::Any,
        },
        ExpectedRoute {
            id: "openai",
            canonical: "openai",
            base_url: "https://api.openai.com/v1",
            models_endpoint: "/models",
            oauth: false,
            auto_max_tokens: true,
            fallback_max_tokens: Some(128_000),
            usage_scope: UsageScope::Any,
        },
        ExpectedRoute {
            id: "deepseek",
            canonical: "deepseek",
            base_url: "https://api.deepseek.com/v1",
            models_endpoint: "/models",
            oauth: false,
            auto_max_tokens: true,
            fallback_max_tokens: Some(384_000),
            usage_scope: UsageScope::Any,
        },
        ExpectedRoute {
            id: "xai",
            canonical: "xai",
            base_url: "https://api.x.ai/v1",
            models_endpoint: "/models",
            oauth: false,
            auto_max_tokens: true,
            fallback_max_tokens: Some(64_000),
            usage_scope: UsageScope::Any,
        },
        ExpectedRoute {
            id: "moonshot",
            canonical: "moonshot",
            base_url: "https://api.moonshot.ai/v1",
            models_endpoint: "/models",
            oauth: false,
            auto_max_tokens: true,
            fallback_max_tokens: Some(131_072),
            usage_scope: UsageScope::Any,
        },
        ExpectedRoute {
            id: "zai",
            canonical: "zai",
            base_url: "https://api.z.ai/api/paas/v4",
            models_endpoint: "",
            oauth: false,
            auto_max_tokens: true,
            fallback_max_tokens: Some(96_000),
            usage_scope: UsageScope::Any,
        },
        ExpectedRoute {
            id: "xai-oauth",
            canonical: "xai",
            base_url: "https://cli-chat-proxy.grok.com/v1",
            models_endpoint: "/models-v2",
            oauth: true,
            auto_max_tokens: true,
            fallback_max_tokens: Some(64_000),
            usage_scope: UsageScope::InteractiveOnly,
        },
        ExpectedRoute {
            id: "moonshot-oauth",
            canonical: "moonshot",
            base_url: "https://api.kimi.com/coding/v1",
            models_endpoint: "/models",
            oauth: true,
            auto_max_tokens: true,
            fallback_max_tokens: Some(64_000),
            usage_scope: UsageScope::InteractiveOnly,
        },
    ];

    for item in expected {
        let actual =
            route::resolve(item.id).unwrap_or_else(|| panic!("route absente: {}", item.id));
        assert_eq!(actual.chat_provider_id, item.id);
        assert_eq!(actual.canonical_provider_id, item.canonical, "{}", item.id);
        assert_eq!(actual.base_url, item.base_url, "{}", item.id);
        assert_eq!(actual.models_endpoint, item.models_endpoint, "{}", item.id);
        assert_eq!(actual.is_oauth(), item.oauth, "{}", item.id);
        assert_eq!(actual.auto_max_tokens, item.auto_max_tokens, "{}", item.id);
        assert_eq!(
            actual.fallback_max_tokens, item.fallback_max_tokens,
            "{}",
            item.id
        );
        assert_eq!(actual.usage_scope, item.usage_scope, "{}", item.id);
    }

    for special_or_unknown in ["codex-oauth", "ollama", "anthropic", "unknown"] {
        assert!(
            route::resolve(special_or_unknown).is_none(),
            "{special_or_unknown}"
        );
    }
}
