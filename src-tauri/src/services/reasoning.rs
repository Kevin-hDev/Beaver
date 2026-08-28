use crate::services::agent_local::types_ollama::OllamaThink;

pub use super::reasoning_effort::{
    codex as codex_effort, openai as openai_effort, openrouter as openrouter_effort,
    simple as simple_effort, zai as zai_effort,
};

pub fn sanitize_mode(mode: Option<String>) -> Option<String> {
    mode.filter(|value| {
        matches!(
            value.as_str(),
            "off" | "auto" | "low" | "medium" | "high" | "xhigh" | "max" | "ultra"
        )
    })
}

fn is_gpt_oss(model: &str) -> bool {
    model.to_lowercase().contains("gpt-oss")
}

pub fn supported_modes(provider: &str, model: &str, supports_thinking: bool) -> Vec<String> {
    if !supports_thinking {
        return Vec::new();
    }
    let provider = crate::services::llm::route::canonical_provider_id(provider);
    crate::services::llm::provider_model_lookup::resolve_reasoning_modes(
        provider,
        model,
        supports_thinking,
    )
}

pub(crate) fn restrict_to_dynamic_modes(
    base: Vec<String>,
    dynamic: Option<&[String]>,
) -> Vec<String> {
    match dynamic {
        Some(dynamic) if !dynamic.is_empty() => base
            .into_iter()
            .filter(|mode| dynamic.contains(mode))
            .collect(),
        _ => base,
    }
}

pub fn provider_model_supports_thinking(provider: &str, model: &str) -> bool {
    let provider = crate::services::llm::route::canonical_provider_id(provider);
    crate::services::llm::provider_model_lookup::resolve_local_or_legacy(provider, model)
        .is_some_and(|resolved| resolved.supports_thinking)
}

pub fn normalize_for_model(
    provider: &str,
    model: &str,
    requested: Option<&str>,
    supports_thinking: bool,
) -> Option<String> {
    let provider = crate::services::llm::route::canonical_provider_id(provider);
    let modes = supported_modes(provider, model, supports_thinking);
    if modes.is_empty() {
        return None;
    }
    if let Some(mode) = requested.filter(|mode| modes.iter().any(|candidate| candidate == mode)) {
        return Some(mode.to_string());
    }
    if provider == "codex-oauth" && model == "gpt-5.3-codex-spark" {
        return Some("high".to_string());
    }
    let preferred =
        crate::services::llm::provider_model_lookup::resolve_local_or_legacy(provider, model)
            .and_then(|resolved| resolved.default_reasoning_mode);
    if preferred
        .as_ref()
        .is_some_and(|mode| modes.iter().any(|candidate| candidate == mode))
    {
        return preferred;
    }
    if modes.iter().any(|mode| mode == "medium") {
        return Some("medium".to_string());
    }
    if modes.iter().any(|mode| mode == "auto") {
        return Some("auto".to_string());
    }
    if let Some(mode) = modes.iter().find(|mode| mode.as_str() != "off") {
        return Some(mode.clone());
    }
    modes.first().cloned()
}

pub fn default_mode(provider: &str, model: &str) -> Option<String> {
    normalize_for_model(
        provider,
        model,
        None,
        provider_model_supports_thinking(provider, model),
    )
}

pub fn enabled(mode: Option<&str>, fallback: bool) -> bool {
    match mode {
        Some("off") => false,
        Some(_) => true,
        None => fallback,
    }
}

pub fn ollama_think(model: &str, mode: Option<&str>, fallback: bool) -> Option<OllamaThink> {
    if is_gpt_oss(model) {
        let effort = match mode {
            Some("low" | "medium" | "high") => mode.unwrap(),
            Some("xhigh") => "high",
            _ => "medium",
        };
        return Some(OllamaThink::Level(effort.to_string()));
    }
    Some(OllamaThink::Bool(enabled(mode, fallback)))
}
