pub const COMPATIBILITY_TOOL_LIMIT: usize = 128;
pub const MISTRAL_TOOL_LIMIT: usize = 128;
pub const DEEPSEEK_TOOL_LIMIT: usize = 128;
pub const XAI_TOOL_LIMIT: usize = 200;
pub const GOOGLE_UPPER_COMPATIBILITY_LIMIT: usize = 512;
pub const LOCAL_RUNTIME_SAFETY_LIMIT: usize = 256;
pub const MAX_CAPACITY_DIAGNOSTIC_ITEMS: usize = 8;

// Strict limits verified on 2026-07-29:
// Mistral: https://docs.mistral.ai/resources/known-limitations
// DeepSeek: https://api-docs.deepseek.com/api/create-chat-completion
// xAI: https://docs.x.ai/developers/tools/function-calling
// Google's 512 cap comes from Vertex and is only an upper compatibility bound here:
// https://cloud.google.com/vertex-ai/generative-ai/docs/multimodal/function-calling
// Other providers use Beaver's conservative cap. Ollama documents no strict cap, so its higher
// value remains an explicit local-runtime safety bound.
pub fn for_policy(policy: crate::services::llm::route_profile::ResolvedToolLimitPolicy) -> usize {
    use crate::services::llm::route_profile::{ToolLimitPolicy, UpstreamToolFamily};

    match policy.policy {
        ToolLimitPolicy::Google => GOOGLE_UPPER_COMPATIBILITY_LIMIT,
        ToolLimitPolicy::Mistral => MISTRAL_TOOL_LIMIT,
        ToolLimitPolicy::DeepSeek => DEEPSEEK_TOOL_LIMIT,
        ToolLimitPolicy::Xai => XAI_TOOL_LIMIT,
        ToolLimitPolicy::Ollama => LOCAL_RUNTIME_SAFETY_LIMIT,
        ToolLimitPolicy::OpenRouterUpstream => match policy.upstream {
            UpstreamToolFamily::Google => GOOGLE_UPPER_COMPATIBILITY_LIMIT,
            UpstreamToolFamily::Xai => XAI_TOOL_LIMIT,
            UpstreamToolFamily::Mistral => MISTRAL_TOOL_LIMIT,
            UpstreamToolFamily::DeepSeek => DEEPSEEK_TOOL_LIMIT,
            UpstreamToolFamily::Other => COMPATIBILITY_TOOL_LIMIT,
        },
        ToolLimitPolicy::Default => COMPATIBILITY_TOOL_LIMIT,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_limits_are_stable_and_model_aware() {
        let limit = |provider, model| {
            let policy =
                crate::services::llm::route_profile::tool_limit_policy(provider, model).unwrap();
            for_policy(policy)
        };
        assert_eq!(limit("mistral", "large"), 128);
        assert_eq!(limit("xai", "grok-4"), 200);
        assert_eq!(limit("google", "gemini-3"), 512);
        assert_eq!(limit("openrouter", "google/gemini-3"), 512);
        assert_eq!(limit("deepseek", "deepseek-chat"), 128);
        assert_eq!(limit("openrouter", "deepseek/deepseek-v3"), 128);
        assert_eq!(limit("openrouter", "unknown/model"), 128);
        assert_eq!(limit("ollama", "qwen3"), 256);
    }
}
