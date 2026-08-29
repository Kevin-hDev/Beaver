#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SchemaPolicy {
    Generic,
    Anthropic,
    Google,
    Kimi,
    Qwen,
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
    AnthropicAutomatic,
    QwenContext,
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
pub(crate) enum ParameterPolicy {
    Default,
    Responses,
    Google,
    Mistral,
    Cerebras,
    OpenRouter,
    DeepSeek,
    Moonshot,
    Xai,
    Zai,
    Ollama,
    Anthropic,
    Qwen,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ErrorPolicy {
    OpenAiCompatible,
    Responses,
    Moonshot,
    Xai,
    XaiOauth,
    Codex,
    Ollama,
}

impl ErrorPolicy {
    pub(crate) const fn max_server_retries(self) -> u32 {
        match self {
            Self::Ollama => 10,
            _ => 0,
        }
    }

    pub(crate) const fn allows_server_retry(self, status: u16, retries: u32) -> bool {
        matches!(status, 500 | 502 | 503 | 504) && retries < self.max_server_retries()
    }
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
    pub gemma4_thinking_guard: bool,
    pub dynamic_reasoning_catalog: bool,
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
    pub parameters: ParameterPolicy,
    pub emit_tool_choice: bool,
    pub tool_stream: bool,
    pub parallel_tool_calls: bool,
    pub upstream_routing: bool,
    pub output_limit_field: &'static str,
}
