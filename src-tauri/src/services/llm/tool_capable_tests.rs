use super::*;

#[test]
fn groq_tool_capable() {
    assert!(supports_tools("groq", "llama-3.3-70b-versatile"));
    assert!(supports_tools("groq", "llama-4-scout-17b-16e-instruct"));
    assert!(!supports_tools("groq", "whisper-large-v3"));
}

#[test]
fn gemini_tool_capable() {
    assert!(supports_tools("google", "gemini-2.5-pro"));
    assert!(supports_tools("google", "gemini-3.5-flash"));
    assert!(supports_tools("google", "gemini-3.1-pro"));
    assert!(supports_tools("google", "gemini-2.5-flash"));
    assert!(supports_tools("google", "gemma-4-31b-it"));
    assert!(supports_tools("google", "gemma-4-26b-a4b-it"));
    assert!(!supports_tools("google", "gemini-2.5-flash-lite"));
    assert!(!supports_tools("google", "text-embedding-004"));
}

#[test]
fn gemini_thinking_capable() {
    assert!(supports_thinking("google", "gemini-2.5-flash"));
    assert!(supports_thinking("google", "gemini-2.5-pro"));
    assert!(supports_thinking("google", "gemini-3.1-pro"));
    assert!(supports_thinking("google", "gemini-3.5-flash"));
    assert!(supports_vision("google", "gemini-3.5-flash"));
}

#[test]
fn mistral_tool_capable() {
    assert!(supports_tools("mistral", "mistral-large-latest"));
    assert!(supports_tools("mistral", "mistral-small-3-24b"));
    assert!(supports_tools("mistral", "codestral-latest"));
    assert!(!supports_tools("mistral", "mistral-embed"));
}

#[test]
fn openai_tool_capable() {
    assert!(supports_tools("openai", "gpt-4o"));
    assert!(supports_tools("openai", "gpt-5.4"));
    assert!(supports_tools("openai", "gpt-5.6-sol"));
    assert!(supports_thinking("openai", "gpt-5.6-terra"));
    assert!(supports_vision("openai", "gpt-5.6-luna"));
    assert!(supports_tools("openai", "o4-mini"));
    assert!(!supports_tools("openai", "text-embedding-3-small"));
}

#[test]
fn org_prefixed_model_ids() {
    assert!(supports_tools(
        "groq",
        "meta-llama/llama-4-scout-17b-16e-instruct"
    ));
    assert!(supports_tools("groq", "qwen/qwen3-32b"));
    assert!(supports_tools(
        "groq",
        "deepseek/deepseek-r1-distill-llama-70b"
    ));
    assert!(!supports_tools("groq", "unknown-org/whisper-large-v3"));
}

#[test]
fn mistral_devstral() {
    assert!(supports_tools("mistral", "devstral-small-latest"));
    assert!(supports_tools("mistral", "magistral-medium-latest"));
    assert!(supports_tools("mistral", "pixtral-large-latest"));
}

#[test]
fn vision_detection_updates() {
    assert!(supports_vision("mistral", "mistral-medium-latest"));
    assert!(supports_vision("mistral", "ministral-3-8b-2512"));
    assert!(supports_vision("google", "gemma-4-31b-it"));
    assert!(supports_vision("google", "gemma-4-26b-a4b-it"));
    assert!(supports_vision("openrouter", "google/gemma-4-31b-it"));
    assert!(supports_vision(
        "openrouter",
        "google/gemma-4-26b-a4b-it:free"
    ));
    assert!(!supports_vision("deepseek", "deepseek-vl"));
}

#[test]
fn unknown_provider_returns_false() {
    assert!(!supports_tools("unknown", "any-model"));
}
