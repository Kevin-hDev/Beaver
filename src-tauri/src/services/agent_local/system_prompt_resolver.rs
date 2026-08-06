use super::system_prompt_store::SystemPromptSettings;
use super::system_prompt_types::{
    PromptMode, PromptOverride, PromptSource, PromptTier, SystemPromptView,
};

pub fn resolve_global(
    settings: &SystemPromptSettings,
    mode: PromptMode,
    tier: PromptTier,
    beaver_prompt: &str,
) -> SystemPromptView {
    match settings.global_override(mode, tier) {
        Some(value) => from_override(value, true, beaver_prompt),
        None => view(beaver_prompt, PromptSource::Beaver, false),
    }
}

pub fn resolve_ollama(
    settings: &SystemPromptSettings,
    model: &str,
    mode: PromptMode,
    tier: PromptTier,
    native_prompt: Option<&str>,
    beaver_prompt: &str,
) -> SystemPromptView {
    if let Some(value) = settings.ollama_override(model, mode, tier) {
        return from_override(value, true, beaver_prompt);
    }
    if let Some(native) = native_prompt.map(str::trim).filter(|value| !value.is_empty()) {
        return view(native, PromptSource::Ollama, false);
    }
    match settings.global_override(mode, tier) {
        Some(value) => from_override(value, false, beaver_prompt),
        None => view(beaver_prompt, PromptSource::Beaver, false),
    }
}

fn from_override(
    value: &PromptOverride,
    customized: bool,
    beaver_prompt: &str,
) -> SystemPromptView {
    match value {
        PromptOverride::Custom(content) => view(content, PromptSource::Custom, customized),
        PromptOverride::Disabled => view("", PromptSource::Custom, customized),
        PromptOverride::Beaver => view(beaver_prompt, PromptSource::Beaver, false),
    }
}

fn view(content: &str, source: PromptSource, customized: bool) -> SystemPromptView {
    SystemPromptView {
        content: content.to_string(),
        source,
        customized,
        disabled: content.is_empty(),
    }
}
