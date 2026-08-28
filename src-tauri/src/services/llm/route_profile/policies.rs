use super::policy_types::*;
use super::types::*;
use crate::services::provider_usage::UsageApiFormat;

pub(super) const AVAILABLE_ANY: RouteAvailability = RouteAvailability {
    interactive: true,
    silent: true,
    automation: true,
    external_channel: true,
    account_metadata: true,
};

pub(super) const INTERACTIVE_ONLY: RouteAvailability = RouteAvailability {
    interactive: true,
    silent: false,
    automation: false,
    external_channel: false,
    account_metadata: true,
};

pub(super) const OPENAI_CHAT_WIRE: WireContract = WireContract {
    family: WireFamily::OpenAiChatCompletions,
    fragments: FragmentMode::DifferentialFragments,
    tool_results: ToolResultPlacement::ToolRole,
    images: ImageFormat::OpenAiNested,
    usage: UsageApiFormat::ChatCompletions,
};

pub(super) const MISTRAL_CHAT_WIRE: WireContract = WireContract {
    images: ImageFormat::MistralFlat,
    ..OPENAI_CHAT_WIRE
};

pub(super) const RESPONSES_WIRE: WireContract = WireContract {
    family: WireFamily::OpenAiResponses,
    fragments: FragmentMode::SemanticEvents,
    tool_results: ToolResultPlacement::ResponsesItem,
    images: ImageFormat::ResponsesInput,
    usage: UsageApiFormat::Responses,
};

pub(super) const OLLAMA_WIRE: WireContract = WireContract {
    family: WireFamily::OllamaNative,
    fragments: FragmentMode::DifferentialFragments,
    tool_results: ToolResultPlacement::OllamaNative,
    images: ImageFormat::OllamaNative,
    usage: UsageApiFormat::ChatCompletions,
};

#[cfg(test)]
pub(super) const ANTHROPIC_WIRE_TEST: WireContract = WireContract {
    family: WireFamily::AnthropicMessages,
    fragments: FragmentMode::SemanticEvents,
    tool_results: ToolResultPlacement::UserToolResultBlock,
    images: ImageFormat::Unsupported,
    usage: UsageApiFormat::ChatCompletions,
};

const fn policy(
    schema: SchemaPolicy,
    cache: CachePolicy,
    parameters: ParameterPolicy,
    errors: ErrorPolicy,
    auth_probe: AuthProbePolicy,
    tool_limits: ToolLimitPolicy,
) -> RoutePolicies {
    RoutePolicies {
        schema,
        cache,
        tool_choice: ToolChoicePolicy::Default,
        parameters,
        errors,
        auth_probe,
        tool_limits,
        include_usage: false,
    }
}

pub(super) const OPENAI_CHAT_DEFAULT: RoutePolicies = policy(
    SchemaPolicy::Generic,
    CachePolicy::None,
    ParameterPolicy::Default,
    ErrorPolicy::OpenAiCompatible,
    AuthProbePolicy::ModelsGet,
    ToolLimitPolicy::Default,
);
pub(super) const GOOGLE: RoutePolicies = RoutePolicies {
    include_usage: true,
    ..policy(
        SchemaPolicy::Google,
        CachePolicy::Google,
        ParameterPolicy::Google,
        ErrorPolicy::OpenAiCompatible,
        AuthProbePolicy::ModelsGet,
        ToolLimitPolicy::Google,
    )
};
pub(super) const MISTRAL: RoutePolicies = policy(
    SchemaPolicy::Generic,
    CachePolicy::PromptKey,
    ParameterPolicy::Mistral,
    ErrorPolicy::OpenAiCompatible,
    AuthProbePolicy::ModelsGet,
    ToolLimitPolicy::Mistral,
);
pub(super) const CEREBRAS: RoutePolicies = RoutePolicies {
    parameters: ParameterPolicy::Cerebras,
    include_usage: true,
    ..OPENAI_CHAT_DEFAULT
};
pub(super) const OPENROUTER: RoutePolicies = policy(
    SchemaPolicy::Upstream,
    CachePolicy::OpenRouter,
    ParameterPolicy::Default,
    ErrorPolicy::OpenAiCompatible,
    AuthProbePolicy::ModelsGet,
    ToolLimitPolicy::OpenRouterUpstream,
);
pub(super) const OPENAI_RESPONSES_PUBLIC: RoutePolicies = RoutePolicies {
    include_usage: true,
    ..policy(
        SchemaPolicy::Generic,
        CachePolicy::OpenAi56,
        ParameterPolicy::Responses,
        ErrorPolicy::Responses,
        AuthProbePolicy::ModelsGet,
        ToolLimitPolicy::Default,
    )
};
pub(super) const DEEPSEEK: RoutePolicies = RoutePolicies {
    include_usage: true,
    tool_choice: ToolChoicePolicy::ProviderNative,
    ..policy(
        SchemaPolicy::Generic,
        CachePolicy::None,
        ParameterPolicy::DeepSeek,
        ErrorPolicy::OpenAiCompatible,
        AuthProbePolicy::ModelsGet,
        ToolLimitPolicy::DeepSeek,
    )
};
pub(super) const XAI_PUBLIC: RoutePolicies = RoutePolicies {
    include_usage: true,
    ..policy(
        SchemaPolicy::Xai,
        CachePolicy::XaiHeader,
        ParameterPolicy::Xai,
        ErrorPolicy::Xai,
        AuthProbePolicy::ModelsGet,
        ToolLimitPolicy::Xai,
    )
};
pub(super) const MOONSHOT: RoutePolicies = RoutePolicies {
    include_usage: true,
    ..policy(
        SchemaPolicy::Kimi,
        CachePolicy::PromptKey,
        ParameterPolicy::Moonshot,
        ErrorPolicy::Moonshot,
        AuthProbePolicy::ModelsGet,
        ToolLimitPolicy::Default,
    )
};
pub(super) const ZAI: RoutePolicies = policy(
    SchemaPolicy::Generic,
    CachePolicy::None,
    ParameterPolicy::Zai,
    ErrorPolicy::OpenAiCompatible,
    AuthProbePolicy::ChatPing,
    ToolLimitPolicy::Default,
);
pub(super) const XAI_OAUTH: RoutePolicies = RoutePolicies {
    include_usage: true,
    ..policy(
        SchemaPolicy::Xai,
        CachePolicy::None,
        ParameterPolicy::Xai,
        ErrorPolicy::XaiOauth,
        AuthProbePolicy::OAuthCatalog,
        ToolLimitPolicy::Xai,
    )
};
pub(super) const CODEX_OAUTH: RoutePolicies = policy(
    SchemaPolicy::Generic,
    CachePolicy::OpenAi56,
    ParameterPolicy::Responses,
    ErrorPolicy::Codex,
    AuthProbePolicy::ClientNative,
    ToolLimitPolicy::Default,
);
pub(super) const OLLAMA_LOCAL: RoutePolicies = policy(
    SchemaPolicy::Generic,
    CachePolicy::None,
    ParameterPolicy::Ollama,
    ErrorPolicy::Ollama,
    AuthProbePolicy::ClientNative,
    ToolLimitPolicy::Ollama,
);
