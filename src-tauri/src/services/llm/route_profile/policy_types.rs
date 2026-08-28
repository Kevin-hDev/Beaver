#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::services::llm) enum SchemaPolicy {
    Generic,
    Google,
    Kimi,
    Xai,
    Upstream,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::services::llm) enum CachePolicy {
    None,
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
pub(in crate::services::llm) enum AuthProbePolicy {
    ModelsGet,
    ChatPing,
    OAuthCatalog,
    ClientNative,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::services::llm) enum ToolLimitPolicy {
    Default,
    Google,
    Mistral,
    DeepSeek,
    Xai,
    OpenRouterUpstream,
    Ollama,
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
}
