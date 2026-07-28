pub const DEFAULT_TOOL_LIMIT: usize = 128;
pub const MISTRAL_TOOL_LIMIT: usize = 128;
pub const GROQ_TOOL_LIMIT: usize = 128;
pub const OPENAI_TOOL_LIMIT: usize = 128;
pub const XAI_TOOL_LIMIT: usize = 200;
pub const GOOGLE_TOOL_LIMIT: usize = 512;
pub const OLLAMA_TOOL_LIMIT: usize = 256;
pub const MAX_CAPACITY_DIAGNOSTIC_ITEMS: usize = 8;

pub fn for_request(provider: &str, model: &str) -> usize {
    match provider {
        "google" | "gemini" => GOOGLE_TOOL_LIMIT,
        "mistral" => MISTRAL_TOOL_LIMIT,
        "groq" => GROQ_TOOL_LIMIT,
        "xai" => XAI_TOOL_LIMIT,
        "ollama" => OLLAMA_TOOL_LIMIT,
        "openai" | "codex-oauth" => OPENAI_TOOL_LIMIT,
        "openrouter" => openrouter_limit(model),
        _ => DEFAULT_TOOL_LIMIT,
    }
}

fn openrouter_limit(model: &str) -> usize {
    let family = model
        .split_once('/')
        .map(|(family, _)| family)
        .unwrap_or_default()
        .to_ascii_lowercase();
    match family.as_str() {
        "google" => GOOGLE_TOOL_LIMIT,
        "x-ai" | "xai" => XAI_TOOL_LIMIT,
        "mistralai" | "mistral" => MISTRAL_TOOL_LIMIT,
        "groq" => GROQ_TOOL_LIMIT,
        "openai" => OPENAI_TOOL_LIMIT,
        _ => DEFAULT_TOOL_LIMIT,
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
        assert_eq!(for_request("openrouter", "unknown/model"), 128);
        assert_eq!(for_request("ollama", "qwen3"), 256);
    }
}
