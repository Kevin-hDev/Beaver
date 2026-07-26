use super::agent_local::types_ollama::OllamaThink;
use super::reasoning::*;

#[test]
fn codex_default_is_medium_and_no_off() {
    assert_eq!(codex_effort("gpt-5.6-sol", None), "medium");
    assert_eq!(codex_effort("gpt-5.6-sol", Some("off")), "medium");
    assert_eq!(codex_effort("gpt-5.6-sol", Some("xhigh")), "xhigh");
}

#[test]
fn codex_effort_rejects_levels_unsupported_by_the_model() {
    assert_eq!(codex_effort("gpt-5.6-sol", Some("ultra")), "ultra");
    assert_eq!(codex_effort("gpt-5.6-terra", Some("max")), "max");
    assert_eq!(codex_effort("gpt-5.6-luna", Some("max")), "max");
    assert_eq!(codex_effort("gpt-5.6-luna", Some("ultra")), "medium");
    assert_eq!(codex_effort("gpt-5.5", Some("max")), "medium");
}

#[test]
fn codex_spark_defaults_to_high_reasoning() {
    assert_eq!(codex_effort("gpt-5.3-codex-spark", None), "high");
    assert_eq!(
        default_mode("codex-oauth", "gpt-5.3-codex-spark").as_deref(),
        Some("high")
    );
}

#[test]
fn gpt_oss_uses_string_effort() {
    let think = ollama_think("gpt-oss:20b", Some("low"), false).unwrap();
    assert_eq!(think, OllamaThink::Level("low".to_string()));
}

#[test]
fn regular_ollama_uses_boolean_thinking() {
    let think = ollama_think("qwen3", Some("off"), true).unwrap();
    assert_eq!(think, OllamaThink::Bool(false));
}

#[test]
fn provider_specific_modes_are_distinct() {
    assert_eq!(
        supported_modes("groq", "openai/gpt-oss-20b", true),
        &["low", "medium", "high"]
    );
    assert_eq!(
        supported_modes("groq", "qwen/qwen3-32b", true),
        &["off", "auto"]
    );
    assert_eq!(
        supported_modes("mistral", "magistral-medium-latest", true),
        &["auto"]
    );
    assert!(supported_modes("mistral", "mistral-small-2506", true).is_empty());
    assert_eq!(
        supported_modes("deepseek", "deepseek-v4-pro", true),
        &["off", "high", "xhigh"]
    );
    assert_eq!(
        supported_modes("google", "gemini-3.5-flash", true),
        &["low", "medium", "high"]
    );
    assert_eq!(
        supported_modes("google", "gemini-2.5-flash", true),
        &["off", "low", "medium", "high"]
    );
    assert_eq!(
        supported_modes("zai", "glm-5.2", true),
        &["off", "auto", "low", "medium", "high", "xhigh"]
    );
    assert_eq!(
        supported_modes("moonshot", "kimi-k2.7-code", true),
        &["auto"]
    );
    assert_eq!(
        supported_modes("moonshot", "k3", true),
        &["low", "high", "max"]
    );
    assert_eq!(
        supported_modes("xai", "grok-4.5", true),
        &["low", "medium", "high"]
    );
    assert_eq!(
        supported_modes("xai", "grok-4.20-0309-reasoning", true),
        &["auto"]
    );
}

#[test]
fn xai_multi_agent_is_detected_as_thinking() {
    assert!(provider_model_supports_thinking(
        "xai",
        "grok-4.20-multi-agent-beta-0309"
    ));
}

#[test]
fn unsupported_model_clears_mode() {
    assert_eq!(
        normalize_for_model("ollama", "gemma4:latest", Some("auto"), false),
        None
    );
}

#[test]
fn switchable_thinking_defaults_to_auto() {
    assert_eq!(
        normalize_for_model("groq", "qwen/qwen3-32b", None, true).as_deref(),
        Some("auto")
    );
}

#[test]
fn adjustable_thinking_without_medium_defaults_to_first_enabled_mode() {
    assert_eq!(
        normalize_for_model("deepseek", "deepseek-v4-pro", None, true).as_deref(),
        Some("high")
    );
}

#[test]
fn explicit_off_mode_is_preserved() {
    assert_eq!(
        normalize_for_model("deepseek", "deepseek-v4-pro", Some("off"), true).as_deref(),
        Some("off")
    );
}

#[test]
fn kimi_k3_defaults_to_max_and_rejects_off() {
    assert_eq!(
        normalize_for_model("moonshot", "k3", None, true).as_deref(),
        Some("max")
    );
    assert_eq!(
        normalize_for_model("moonshot", "k3", Some("off"), true).as_deref(),
        Some("max")
    );
}

#[test]
fn kimi_oauth_preserves_every_supported_k3_effort() {
    assert!(provider_model_supports_thinking("moonshot-oauth", "k3"));
    assert_eq!(
        supported_modes("moonshot-oauth", "k3", true),
        &["low", "high", "max"]
    );
    for effort in ["low", "high", "max"] {
        assert_eq!(
            normalize_for_model("moonshot-oauth", "k3", Some(effort), true).as_deref(),
            Some(effort)
        );
    }
    assert_eq!(
        normalize_for_model("moonshot-oauth", "k3", None, true).as_deref(),
        Some("max")
    );
}
