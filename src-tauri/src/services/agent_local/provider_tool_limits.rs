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
pub fn for_request(provider: &str, model: &str) -> usize {
    match provider {
        "google" | "gemini" => GOOGLE_UPPER_COMPATIBILITY_LIMIT,
        "mistral" => MISTRAL_TOOL_LIMIT,
        "deepseek" => DEEPSEEK_TOOL_LIMIT,
        "xai" => XAI_TOOL_LIMIT,
        "ollama" => LOCAL_RUNTIME_SAFETY_LIMIT,
        "openrouter" => openrouter_limit(model),
        _ => COMPATIBILITY_TOOL_LIMIT,
    }
}

fn openrouter_limit(model: &str) -> usize {
    let family = model
        .split_once('/')
        .map(|(family, _)| family)
        .unwrap_or_default()
        .to_ascii_lowercase();
    match family.as_str() {
        "google" => GOOGLE_UPPER_COMPATIBILITY_LIMIT,
        "x-ai" | "xai" => XAI_TOOL_LIMIT,
        "mistralai" | "mistral" => MISTRAL_TOOL_LIMIT,
        "deepseek" => DEEPSEEK_TOOL_LIMIT,
        _ => COMPATIBILITY_TOOL_LIMIT,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_limits_are_stable_and_model_aware() {
        assert_eq!(for_request("mistral", "large"), 128);
        assert_eq!(for_request("xai", "grok-4"), 200);
        assert_eq!(for_request("google", "gemini-3"), 512);
        assert_eq!(for_request("openrouter", "google/gemini-3"), 512);
        assert_eq!(for_request("deepseek", "deepseek-chat"), 128);
        assert_eq!(
            for_request("openrouter", "deepseek/deepseek-v3"),
            128
        );
        assert_eq!(for_request("openrouter", "unknown/model"), 128);
        assert_eq!(for_request("ollama", "qwen3"), 256);
    }
}
