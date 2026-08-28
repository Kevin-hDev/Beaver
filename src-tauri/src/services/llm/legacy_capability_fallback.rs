pub(super) struct LegacyCapabilities {
    pub supports_tools: bool,
    pub supports_vision: bool,
    pub supports_thinking: bool,
    pub reasoning_modes: Vec<String>,
    pub default_reasoning_mode: Option<String>,
}

pub(super) fn resolve(provider: &str, model: &str) -> LegacyCapabilities {
    let supports_thinking = if provider == "ollama" {
        is_gpt_oss(model)
    } else {
        super::tool_capable::supports_thinking(provider, model)
    };
    LegacyCapabilities {
        supports_tools: super::tool_capable::supports_tools(provider, model),
        supports_vision: super::tool_capable::supports_vision(provider, model),
        supports_thinking,
        reasoning_modes: if supports_thinking {
            reasoning_modes(provider, model)
        } else {
            Vec::new()
        },
        default_reasoning_mode: default_reasoning_mode(provider, model),
    }
}

pub(super) fn reasoning_modes(provider: &str, model: &str) -> Vec<String> {
    let modes: &[&str] = match provider {
        "codex-oauth" if matches!(model, "gpt-5.6-sol" | "gpt-5.6-terra") => {
            &["low", "medium", "high", "xhigh", "max", "ultra"]
        }
        "codex-oauth" if model == "gpt-5.6-luna" => &["low", "medium", "high", "xhigh", "max"],
        "codex-oauth" => &["low", "medium", "high", "xhigh"],
        "ollama" if is_gpt_oss(model) => &["low", "medium", "high"],
        "ollama" => &["off", "auto"],
        "openai" if super::providers::openai::is_gpt_56(model) => {
            &["off", "low", "medium", "high", "xhigh", "max"]
        }
        "openai" => &["off", "low", "medium", "high", "xhigh"],
        "openrouter" if super::providers::openai::is_gpt_56(model) => {
            &["off", "low", "medium", "high", "xhigh", "max"]
        }
        "openrouter" if model.to_lowercase().ends_with("grok-4.5") => &["low", "medium", "high"],
        "openrouter" => &["off", "auto", "low", "medium", "high", "xhigh"],
        "google" => crate::services::reasoning_google::supported_modes(model),
        "deepseek" => &["off", "high", "xhigh"],
        "mistral" if super::providers::mistral::is_adjustable_reasoning(&lower(model)) => {
            &["off", "high"]
        }
        "mistral" => &[],
        "cerebras" if is_gpt_oss(model) => &["off", "low", "medium", "high"],
        "cerebras" => &["off", "auto"],
        "moonshot" if super::providers::moonshot::is_k3(&lower(model)) => &["low", "high", "max"],
        "moonshot" if super::providers::moonshot::is_forced_thinking(&lower(model)) => &["auto"],
        "moonshot" => &["off", "auto"],
        "xai" => super::providers::xai::reasoning_modes(model),
        "zai" if model.to_lowercase().starts_with("glm-5.2") => {
            &["off", "auto", "low", "medium", "high", "xhigh"]
        }
        "zai" => &["off", "auto"],
        _ => &["off", "auto"],
    };
    modes.iter().map(|mode| (*mode).to_string()).collect()
}

pub(super) fn default_reasoning_mode(provider: &str, model: &str) -> Option<String> {
    if provider == "codex-oauth" && model == "gpt-5.3-codex-spark" {
        return Some("high".to_string());
    }
    (provider == "moonshot")
        .then(|| super::providers::moonshot::default_reasoning_mode(&lower(model)))
        .flatten()
        .map(str::to_string)
}

fn is_gpt_oss(model: &str) -> bool {
    model.to_lowercase().contains("gpt-oss")
}

fn lower(model: &str) -> String {
    model.to_lowercase()
}
