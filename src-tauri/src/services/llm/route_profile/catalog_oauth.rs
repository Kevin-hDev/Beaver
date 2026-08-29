use super::policies::*;
use super::types::*;
use crate::services::llm_oauth::LlmOAuthProvider;
use crate::services::reasoning_continuity::contract::RouteId;

const VERIFIED_AT: &str = "2026-08-28";
const SOURCE: &str = "docs/providers";
const fn oauth(provider: LlmOAuthProvider) -> AuthKind {
    AuthKind::OAuth {
        provider,
        source: SOURCE,
        verified_at: VERIFIED_AT,
    }
}
const fn endpoint(base_url: &'static str, models_endpoint: &'static str) -> EndpointPolicy {
    EndpointPolicy::Static {
        base_url,
        models_endpoint,
    }
}
const OUTPUT: OutputLimitPolicy = OutputLimitPolicy {
    automatic: true,
    fallback: Some(64_000),
};

pub(super) const OAUTH_PROFILES: &[RouteProfile] = &[
    RouteProfile {
        id: RouteId::XaiOauth,
        canonical_provider: CanonicalProviderId::Xai,
        display_name: "xAI",
        client: ClientSelector::XaiOauth,
        wire: RESPONSES_WIRE,
        auth: oauth(LlmOAuthProvider::Xai),
        endpoint: endpoint(crate::services::llm_oauth::XAI_PROXY_BASE_URL, "/models-v2"),
        availability: INTERACTIVE_ONLY,
        catalog: CatalogPolicy::Hidden,
        strict_model_allowlist: false,
        output_limits: OUTPUT,
        policies: XAI_OAUTH,
    },
    RouteProfile {
        id: RouteId::MoonshotOauth,
        canonical_provider: CanonicalProviderId::Moonshot,
        display_name: "Moonshot AI",
        client: ClientSelector::OpenAiCompat,
        wire: OPENAI_CHAT_WIRE,
        auth: oauth(LlmOAuthProvider::Kimi),
        endpoint: endpoint("https://api.kimi.com/coding/v1", "/models"),
        availability: INTERACTIVE_ONLY,
        catalog: CatalogPolicy::Hidden,
        strict_model_allowlist: false,
        output_limits: OUTPUT,
        policies: MOONSHOT,
    },
    RouteProfile {
        id: RouteId::CodexOauth,
        canonical_provider: CanonicalProviderId::CodexOauth,
        display_name: "Codex",
        client: ClientSelector::Codex,
        wire: RESPONSES_WIRE,
        auth: AuthKind::ClientOAuth {
            credential_id: "codex-oauth",
            source: "client-owned OAuth",
            verified_at: VERIFIED_AT,
        },
        endpoint: EndpointPolicy::ConnectionConfigured,
        availability: AVAILABLE_ANY,
        catalog: CatalogPolicy::Hidden,
        strict_model_allowlist: false,
        output_limits: OutputLimitPolicy {
            automatic: true,
            fallback: Some(128_000),
        },
        policies: CODEX_OAUTH,
    },
];
