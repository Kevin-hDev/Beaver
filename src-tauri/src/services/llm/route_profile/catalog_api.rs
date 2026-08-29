use super::policies::*;
use super::types::*;
use crate::services::reasoning_continuity::contract::RouteId;

const VERIFIED_AT: &str = "2026-08-28";
const SOURCE: &str = "docs/providers";

const fn api_key(id: &'static str) -> AuthKind {
    AuthKind::ApiKey {
        credential_id: id,
        header: ApiKeyHeader::Bearer,
        headers: &[],
        source: SOURCE,
        verified_at: VERIFIED_AT,
    }
}
const fn anthropic_api_key() -> AuthKind {
    AuthKind::ApiKey {
        credential_id: "anthropic",
        header: ApiKeyHeader::XApiKey,
        headers: &[("anthropic-version", "2023-06-01")],
        source: SOURCE,
        verified_at: "2026-08-29",
    }
}
const fn endpoint(base_url: &'static str, models_endpoint: &'static str) -> EndpointPolicy {
    EndpointPolicy::Static {
        base_url,
        models_endpoint,
    }
}
const fn public(signup_url: &'static str) -> CatalogPolicy {
    CatalogPolicy::PublicApi { signup_url }
}
const fn configurable(signup_url: &'static str) -> CatalogPolicy {
    CatalogPolicy::ConfigurableApi { signup_url }
}
const fn limits(automatic: bool, fallback: Option<u32>) -> OutputLimitPolicy {
    OutputLimitPolicy {
        automatic,
        fallback,
    }
}

pub(super) const API_PROFILES: &[RouteProfile] = &[
    RouteProfile {
        id: RouteId::Google,
        canonical_provider: CanonicalProviderId::Google,
        display_name: "Google Gemini",
        client: ClientSelector::OpenAiCompat,
        wire: OPENAI_CHAT_WIRE,
        auth: api_key("google"),
        endpoint: endpoint(
            "https://generativelanguage.googleapis.com/v1beta/openai",
            "/models",
        ),
        availability: AVAILABLE_ANY,
        catalog: public("https://aistudio.google.com/app/apikey"),
        output_limits: limits(true, None),
        policies: GOOGLE,
    },
    RouteProfile {
        id: RouteId::Mistral,
        canonical_provider: CanonicalProviderId::Mistral,
        display_name: "Mistral",
        client: ClientSelector::OpenAiCompat,
        wire: MISTRAL_CHAT_WIRE,
        auth: api_key("mistral"),
        endpoint: endpoint("https://api.mistral.ai/v1", "/models"),
        availability: AVAILABLE_ANY,
        catalog: public("https://console.mistral.ai/api-keys"),
        output_limits: limits(true, Some(64_000)),
        policies: MISTRAL,
    },
    RouteProfile {
        id: RouteId::Cerebras,
        canonical_provider: CanonicalProviderId::Cerebras,
        display_name: "Cerebras",
        client: ClientSelector::OpenAiCompat,
        wire: OPENAI_CHAT_WIRE,
        auth: api_key("cerebras"),
        endpoint: endpoint("https://api.cerebras.ai/v1", "/models"),
        availability: AVAILABLE_ANY,
        catalog: public("https://cloud.cerebras.ai/"),
        output_limits: limits(false, None),
        policies: CEREBRAS,
    },
    RouteProfile {
        id: RouteId::OpenRouter,
        canonical_provider: CanonicalProviderId::OpenRouter,
        display_name: "OpenRouter",
        client: ClientSelector::OpenAiCompat,
        wire: OPENAI_CHAT_WIRE,
        auth: api_key("openrouter"),
        endpoint: endpoint("https://openrouter.ai/api/v1", "/models"),
        availability: AVAILABLE_ANY,
        catalog: public("https://openrouter.ai/settings/keys"),
        output_limits: limits(true, Some(64_000)),
        policies: OPENROUTER,
    },
    RouteProfile {
        id: RouteId::OpenAi,
        canonical_provider: CanonicalProviderId::OpenAi,
        display_name: "OpenAI",
        client: ClientSelector::OpenAiResponses,
        wire: RESPONSES_WIRE,
        auth: api_key("openai"),
        endpoint: endpoint("https://api.openai.com/v1", "/models"),
        availability: AVAILABLE_ANY,
        catalog: public("https://platform.openai.com/api-keys"),
        output_limits: limits(true, Some(128_000)),
        policies: OPENAI_RESPONSES_PUBLIC,
    },
    RouteProfile {
        id: RouteId::DeepSeek,
        canonical_provider: CanonicalProviderId::DeepSeek,
        display_name: "DeepSeek",
        client: ClientSelector::OpenAiCompat,
        wire: OPENAI_CHAT_WIRE,
        auth: api_key("deepseek"),
        endpoint: endpoint("https://api.deepseek.com/v1", "/models"),
        availability: AVAILABLE_ANY,
        catalog: public("https://platform.deepseek.com/api_keys"),
        output_limits: limits(true, Some(384_000)),
        policies: DEEPSEEK,
    },
    RouteProfile {
        id: RouteId::Xai,
        canonical_provider: CanonicalProviderId::Xai,
        display_name: "xAI",
        client: ClientSelector::OpenAiResponses,
        wire: RESPONSES_WIRE,
        auth: api_key("xai"),
        endpoint: endpoint("https://api.x.ai/v1", "/models"),
        availability: AVAILABLE_ANY,
        catalog: public("https://console.x.ai"),
        output_limits: limits(true, Some(64_000)),
        policies: XAI_PUBLIC,
    },
    RouteProfile {
        id: RouteId::Moonshot,
        canonical_provider: CanonicalProviderId::Moonshot,
        display_name: "Moonshot Kimi",
        client: ClientSelector::OpenAiCompat,
        wire: OPENAI_CHAT_WIRE,
        auth: api_key("moonshot"),
        endpoint: endpoint("https://api.moonshot.ai/v1", "/models"),
        availability: AVAILABLE_ANY,
        catalog: public("https://platform.kimi.ai/console/api-keys"),
        output_limits: limits(true, Some(131_072)),
        policies: MOONSHOT,
    },
    RouteProfile {
        id: RouteId::Zai,
        canonical_provider: CanonicalProviderId::Zai,
        display_name: "Z.ai GLM",
        client: ClientSelector::OpenAiCompat,
        wire: OPENAI_CHAT_WIRE,
        auth: api_key("zai"),
        endpoint: endpoint("https://api.z.ai/api/paas/v4", ""),
        availability: AVAILABLE_ANY,
        catalog: public("https://z.ai/manage-apikey/apikey-list"),
        output_limits: limits(true, Some(96_000)),
        policies: ZAI,
    },
    RouteProfile {
        id: RouteId::Anthropic,
        canonical_provider: CanonicalProviderId::Anthropic,
        display_name: "Anthropic Claude",
        client: ClientSelector::Anthropic,
        wire: ANTHROPIC_WIRE,
        auth: anthropic_api_key(),
        endpoint: endpoint("https://api.anthropic.com/v1", "/models"),
        availability: CANDIDATE_ONLY,
        catalog: configurable("https://console.anthropic.com/settings/keys"),
        output_limits: limits(true, Some(64_000)),
        policies: ANTHROPIC,
    },
    RouteProfile {
        id: RouteId::Qwen,
        canonical_provider: CanonicalProviderId::Qwen,
        display_name: "Qwen",
        client: ClientSelector::OpenAiCompat,
        wire: OPENAI_CHAT_WIRE,
        auth: api_key("qwen"),
        endpoint: EndpointPolicy::ProviderConnection {
            resolver: ConnectionEndpointResolver::QwenModelStudio,
        },
        availability: CANDIDATE_ONLY,
        catalog: configurable("https://modelstudio.console.alibabacloud.com/"),
        output_limits: limits(true, Some(131_072)),
        policies: OPENAI_CHAT_DEFAULT,
    },
];
