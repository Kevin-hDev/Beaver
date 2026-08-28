use crate::services::llm_oauth::LlmOAuthProvider;
use crate::services::provider_usage::UsageApiFormat;
use crate::services::reasoning_continuity::contract::RouteId;

use super::policy_types::RoutePolicies;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(
    dead_code,
    reason = "Anthropic is a compile-time candidate, not an active route"
)]
pub(in crate::services::llm) enum ClientSelector {
    OpenAiCompat,
    OpenAiResponses,
    XaiOauth,
    Codex,
    OllamaLocal,
    Anthropic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::services::llm) enum CanonicalProviderId {
    Ollama,
    Google,
    Mistral,
    Cerebras,
    OpenRouter,
    OpenAi,
    DeepSeek,
    Xai,
    Moonshot,
    Zai,
    CodexOauth,
}

impl CanonicalProviderId {
    pub(in crate::services::llm) const fn as_str(self) -> &'static str {
        match self {
            Self::Ollama => "ollama",
            Self::Google => "google",
            Self::Mistral => "mistral",
            Self::Cerebras => "cerebras",
            Self::OpenRouter => "openrouter",
            Self::OpenAi => "openai",
            Self::DeepSeek => "deepseek",
            Self::Xai => "xai",
            Self::Moonshot => "moonshot",
            Self::Zai => "zai",
            Self::CodexOauth => "codex-oauth",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(
    dead_code,
    reason = "candidate wire families are required before route activation"
)]
pub(in crate::services::llm) enum WireFamily {
    OpenAiChatCompletions,
    OpenAiResponses,
    OllamaNative,
    AnthropicMessages,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::services::llm) enum FragmentMode {
    SemanticEvents,
    DifferentialFragments,
    CumulativeFragments,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(
    dead_code,
    reason = "candidate tool placements are required before route activation"
)]
pub(in crate::services::llm) enum ToolResultPlacement {
    ToolRole,
    ResponsesItem,
    UserToolResultBlock,
    OllamaNative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(
    dead_code,
    reason = "candidate routes must declare explicit image refusal"
)]
pub(in crate::services::llm) enum ImageFormat {
    OpenAiNested,
    MistralFlat,
    ResponsesInput,
    OllamaNative,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::services::llm) struct WireContract {
    pub family: WireFamily,
    pub fragments: FragmentMode,
    pub tool_results: ToolResultPlacement,
    pub images: ImageFormat,
    pub usage: UsageApiFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(
    dead_code,
    reason = "native API-key headers are reserved for candidate routes"
)]
pub(in crate::services::llm) enum ApiKeyHeader {
    Bearer,
    XApiKey,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::services::llm) enum AuthKind {
    ApiKey {
        credential_id: &'static str,
        header: ApiKeyHeader,
        source: &'static str,
        verified_at: &'static str,
    },
    OAuth {
        provider: LlmOAuthProvider,
        source: &'static str,
        verified_at: &'static str,
    },
    ClientOAuth {
        credential_id: &'static str,
        source: &'static str,
        verified_at: &'static str,
    },
    Local,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::services::llm) enum EndpointPolicy {
    Static {
        base_url: &'static str,
        models_endpoint: &'static str,
    },
    ConnectionConfigured,
    OllamaLocal,
}

impl EndpointPolicy {
    pub(in crate::services::llm) const fn static_parts(
        self,
    ) -> Option<(&'static str, &'static str)> {
        match self {
            Self::Static {
                base_url,
                models_endpoint,
            } => Some((base_url, models_endpoint)),
            Self::ConnectionConfigured | Self::OllamaLocal => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::services::llm) struct RouteAvailability {
    pub interactive: bool,
    pub silent: bool,
    pub automation: bool,
    pub external_channel: bool,
    pub account_metadata: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::services::llm) enum CatalogPolicy {
    PublicApi { signup_url: &'static str },
    Hidden,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::services::llm) struct OutputLimitPolicy {
    pub automatic: bool,
    pub fallback: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::services::llm) struct RouteProfile {
    pub id: RouteId,
    pub canonical_provider: CanonicalProviderId,
    pub display_name: &'static str,
    pub client: ClientSelector,
    pub wire: WireContract,
    pub auth: AuthKind,
    pub endpoint: EndpointPolicy,
    pub availability: RouteAvailability,
    pub catalog: CatalogPolicy,
    pub output_limits: OutputLimitPolicy,
    pub policies: RoutePolicies,
}
