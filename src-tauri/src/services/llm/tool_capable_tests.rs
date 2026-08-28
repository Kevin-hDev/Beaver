use super::*;

#[test]
fn gemini_tool_capable() {
    assert!(supports_tools("google", "gemini-2.5-pro"));
    assert!(supports_tools("google", "gemini-3.5-flash"));
    assert!(supports_tools("google", "gemini-3.1-pro"));
    assert!(supports_tools("google", "gemini-2.5-flash"));
    assert!(supports_tools("google", "gemma-4-31b-it"));
    assert!(supports_tools("google", "gemma-4-26b-a4b-it"));
    assert!(supports_tools("google", "gemini-2.5-flash-lite"));
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
    assert!(supports_tools("mistral", "labs-leanstral-1-5"));
    assert!(!supports_tools("mistral", "mistral-embed"));
}

#[test]
fn cerebras_current_models_keep_their_capabilities() {
    assert!(supports_tools("cerebras", "zai-glm-4.7"));
    assert!(supports_tools("cerebras", "gemma-4-31b"));
    assert!(supports_thinking("cerebras", "gpt-oss-120b"));
    assert!(supports_vision("cerebras", "gemma-4-31b"));
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
fn vision_detection_updates() {
    assert!(supports_vision("mistral", "mistral-medium-latest"));
    assert!(supports_vision("mistral", "ministral-8b-2512"));
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
