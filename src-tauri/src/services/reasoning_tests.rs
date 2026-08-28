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
        supported_modes("mistral", "mistral-medium-3", true),
        &["off", "high"]
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
        supported_modes("zai", "glm-5.3", true),
        &["low", "high", "max"]
    );
    assert_eq!(
        supported_modes("xai", "grok-4.6", true),
        &["low", "medium", "high", "xhigh"]
    );
    assert_eq!(
        supported_modes("google", "gemini-3.7-flash", true),
        &["low", "medium", "high"]
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
fn new_models_use_the_registry_default_and_reject_unsupported_off() {
    for (provider, model, expected) in [
        ("zai", "glm-5.3", "max"),
        ("xai", "grok-4.6", "high"),
        ("google", "gemini-3.7-flash", "medium"),
    ] {
        assert_eq!(
            normalize_for_model(provider, model, None, true).as_deref(),
            Some(expected)
        );
        assert_eq!(
            normalize_for_model(provider, model, Some("off"), true).as_deref(),
            Some(expected)
        );
    }
}

#[test]
fn grok_45_keeps_its_previous_medium_default() {
    assert_eq!(
        normalize_for_model("xai", "grok-4.5", None, true).as_deref(),
        Some("medium")
    );
}

#[test]
fn dynamic_reasoning_levels_can_only_restrict_the_static_registry() {
    let base = ["low", "medium", "high", "xhigh"]
        .into_iter()
        .map(str::to_string)
        .collect();
    let dynamic = ["low", "medium", "high"]
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();

    assert_eq!(
        restrict_to_dynamic_modes(base, Some(&dynamic)),
        ["low", "medium", "high"]
    );
    assert!(restrict_to_dynamic_modes(vec!["high".into()], Some(&["low".into()])).is_empty());
}

#[test]
fn supported_modes_and_default_use_validated_runtime_restrictions() {
    let model = crate::services::llm::types::ModelInfo {
        id: "dynamic-model".into(),
        display_name: None,
        owned_by: None,
        context_length: Some(32_000),
        max_output_tokens: None,
        supports_tools: true,
        supports_vision: false,
        supports_thinking: true,
        supports_fast_mode: false,
        reasoning_modes: vec!["auto".into()],
        default_reasoning_mode: Some("auto".into()),
        is_free: false,
    };
    crate::services::llm::runtime_models::replace_provider("dynamic-fixture", &[model]);

    assert_eq!(
        supported_modes("dynamic-fixture", "dynamic-model", true),
        ["auto"]
    );
    assert_eq!(
        normalize_for_model("dynamic-fixture", "dynamic-model", None, true).as_deref(),
        Some("auto")
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
        normalize_for_model("ollama", "qwen3.5:4b", None, true).as_deref(),
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
