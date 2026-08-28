#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SchemaPolicy {
    Generic,
    Google,
    Kimi,
    Xai,
    Upstream,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExtensionToolPolicy {
    All,
    WithoutExtensions,
    NoTools,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ResolvedToolPolicy {
    pub schema: SchemaPolicy,
    pub strict: bool,
    pub extensions: ExtensionToolPolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CachePolicy {
    None,
    Google,
    OpenAi56,
    OpenRouter,
    PromptKey,
    XaiHeader,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code, reason = "closed variants are reserved for route migration")]
pub(in crate::services::llm) enum ToolChoicePolicy {
    Default,
    ProviderNative,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::services::llm) enum ParameterPolicy {
    Default,
    Responses,
    Google,
    Mistral,
    Cerebras,
    DeepSeek,
    Moonshot,
    Xai,
    Zai,
    Ollama,
    #[allow(dead_code, reason = "Anthropic remains a compile-time candidate")]
    Anthropic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::services::llm) enum ErrorPolicy {
    OpenAiCompatible,
    Responses,
    XaiOauth,
    Codex,
    Ollama,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code, reason = "missing probes are explicit in candidate tests")]
pub(crate) enum AuthProbePolicy {
    ModelsGet,
    ChatPing,
    OAuthCatalog,
    ClientNative,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ToolLimitPolicy {
    Default,
    Google,
    Mistral,
    DeepSeek,
    Xai,
    OpenRouterUpstream,
    Ollama,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UpstreamToolFamily {
    Google,
    Xai,
    Mistral,
    DeepSeek,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ResolvedToolLimitPolicy {
    pub policy: ToolLimitPolicy,
    pub upstream: UpstreamToolFamily,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::services::llm) struct RoutePolicies {
    pub schema: SchemaPolicy,
    pub cache: CachePolicy,
    pub tool_choice: ToolChoicePolicy,
    pub parameters: ParameterPolicy,
    pub errors: ErrorPolicy,
    pub auth_probe: AuthProbePolicy,
    pub tool_limits: ToolLimitPolicy,
    pub include_usage: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ResolvedCachePolicy<'a> {
    pub route_id: &'static str,
    pub model: &'a str,
    pub kind: CachePolicy,
    pub include_usage: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ResolvedPayloadPolicy {
    pub message: super::types::MessageWirePolicy,
    pub emit_tool_choice: bool,
    pub tool_stream: bool,
    pub upstream_routing: bool,
    pub output_limit_field: &'static str,
}
