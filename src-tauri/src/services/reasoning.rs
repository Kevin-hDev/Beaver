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

fn lower(model: &str) -> String {
    model.to_lowercase()
}

fn is_zai_effort_reasoning(model: &str) -> bool {
    model.to_lowercase().starts_with("glm-5.2")
}

pub fn supported_modes(provider: &str, model: &str, supports_thinking: bool) -> Vec<String> {
    if !supports_thinking {
        return Vec::new();
    }
    let provider = crate::services::llm::route::canonical_provider_id(provider);
    let base = if let Some(reasoning) =
        crate::services::llm::provider_model_lookup::local_reasoning(provider, model)
    {
        if !reasoning.modes.is_empty() {
            reasoning.modes
        } else {
            fallback_supported_modes(provider, model)
                .iter()
                .map(|mode| (*mode).to_string())
                .collect()
        }
    } else {
        fallback_supported_modes(provider, model)
            .iter()
            .map(|mode| (*mode).to_string())
            .collect()
    };
    let dynamic = crate::services::llm::runtime_models::lookup(provider, model)
        .map(|entry| entry.reasoning_modes);
    restrict_to_dynamic_modes(base, dynamic.as_deref())
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

fn fallback_supported_modes(provider: &str, model: &str) -> &'static [&'static str] {
    match provider {
        "codex-oauth" if model == "gpt-5.6-sol" || model == "gpt-5.6-terra" => {
            &["low", "medium", "high", "xhigh", "max", "ultra"]
        }
        "codex-oauth" if model == "gpt-5.6-luna" => &["low", "medium", "high", "xhigh", "max"],
        "codex-oauth" => &["low", "medium", "high", "xhigh"],
        "ollama" if is_gpt_oss(model) => &["low", "medium", "high"],
        "ollama" => &["off", "auto"],
        "openai" if crate::services::llm::providers::openai::is_gpt_56(model) => {
            &["off", "low", "medium", "high", "xhigh", "max"]
        }
        "openai" => &["off", "low", "medium", "high", "xhigh"],
        "openrouter" if crate::services::llm::providers::openai::is_gpt_56(model) => {
            &["off", "low", "medium", "high", "xhigh", "max"]
        }
        "openrouter" if model.to_lowercase().ends_with("grok-4.5") => &["low", "medium", "high"],
        "openrouter" => &["off", "auto", "low", "medium", "high", "xhigh"],
        "google" => crate::services::reasoning_google::supported_modes(model),
        "groq" if crate::services::llm::providers::groq::is_gpt_oss_effort(&lower(model)) => {
            &["low", "medium", "high"]
        }
        "groq" if crate::services::llm::providers::groq::is_qwen_switchable(&lower(model)) => {
            &["off", "auto"]
        }
        "groq" => &["auto"],
        "deepseek" => &["off", "high", "xhigh"],
        "mistral"
            if crate::services::llm::providers::mistral::is_adjustable_reasoning(&lower(model)) =>
        {
            &["off", "high"]
        }
        "mistral" => &[],
        "cerebras" if is_gpt_oss(model) => &["low", "medium", "high"],
        "cerebras" => &["auto"],
        "moonshot" if crate::services::llm::providers::moonshot::is_k3(&lower(model)) => {
            &["low", "high", "max"]
        }
        "moonshot"
            if crate::services::llm::providers::moonshot::is_forced_thinking(&lower(model)) =>
        {
            &["auto"]
        }
        "moonshot" => &["off", "auto"],
        "xai" => crate::services::llm::providers::xai::reasoning_modes(model),
        "zai" if is_zai_effort_reasoning(model) => {
            &["off", "auto", "low", "medium", "high", "xhigh"]
        }
        "zai" => &["off", "auto"],
        _ => &["off", "auto"],
    }
}

pub fn provider_model_supports_thinking(provider: &str, model: &str) -> bool {
    let provider = crate::services::llm::route::canonical_provider_id(provider);
    match provider {
        "codex-oauth" => true,
        "ollama" => is_gpt_oss(model),
        "openai" => {
            let model = model.to_lowercase();
            model.starts_with("o3") || model.starts_with("o4") || model.starts_with("gpt-5")
        }
        "deepseek" | "groq" | "google" | "openrouter" | "mistral" | "cerebras" | "xai"
        | "moonshot" | "zai" => {
            crate::services::llm::tool_capable::supports_thinking(provider, model)
        }
        _ => false,
    }
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
    let preferred = crate::services::llm::runtime_models::lookup(provider, model)
        .and_then(|entry| entry.default_reasoning_mode)
        .or_else(|| {
            crate::services::llm::provider_model_lookup::local_reasoning(provider, model)
                .and_then(|reasoning| reasoning.default_mode)
        })
        .or_else(|| {
            (provider == "moonshot").then(|| {
                crate::services::llm::providers::moonshot::default_reasoning_mode(&lower(model))
                    .map(str::to_string)
            })?
        });
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
