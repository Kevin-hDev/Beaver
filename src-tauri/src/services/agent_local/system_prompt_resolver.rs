use super::system_prompt_store::SystemPromptSettings;
use super::system_prompt_types::{
    PromptMode, PromptOverride, PromptSelection, PromptSource, PromptTier, SystemPromptView,
};

pub fn resolve_global(
    settings: &SystemPromptSettings,
    mode: PromptMode,
    tier: PromptTier,
    beaver_prompt: &str,
) -> SystemPromptView {
    match settings.global_override(mode, tier) {
        Some(value) => from_override(value, None),
        None => view(beaver_prompt, PromptSource::Beaver, PromptSelection::Default, None),
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
    let native_prompt = native_prompt.map(str::trim).filter(|value| !value.is_empty());
    let native_available = native_prompt.is_some();
    if let Some(value) = settings.ollama_override(model, mode, tier) {
        return from_override(value, Some(native_available));
    }
    if settings.ollama_uses_beaver(model, mode, tier) {
        return view(
            beaver_prompt,
            PromptSource::Beaver,
            PromptSelection::Beaver,
            Some(native_available),
        );
    }
    if let Some(native) = native_prompt {
        return view(native, PromptSource::Ollama, PromptSelection::Default, Some(true));
    }
    match settings.global_override(mode, tier) {
        Some(PromptOverride::Custom(content)) => view(
            content,
            PromptSource::Custom,
            PromptSelection::Default,
            Some(false),
        ),
        Some(PromptOverride::Disabled) => {
            view("", PromptSource::Custom, PromptSelection::Default, Some(false))
        }
        None => view(
            beaver_prompt,
            PromptSource::Beaver,
            PromptSelection::Default,
            Some(false),
        ),
    }
}

pub fn resolve_ollama_without_native(
    settings: &SystemPromptSettings,
    model: &str,
    mode: PromptMode,
    tier: PromptTier,
    beaver_prompt: &str,
) -> Option<SystemPromptView> {
    if let Some(value) = settings.ollama_override(model, mode, tier) {
        return Some(from_override(value, None));
    }
    settings.ollama_uses_beaver(model, mode, tier).then(|| {
        view(
            beaver_prompt,
            PromptSource::Beaver,
            PromptSelection::Beaver,
            None,
        )
    })
}

fn from_override(
    value: &PromptOverride,
    native_prompt_available: Option<bool>,
) -> SystemPromptView {
    match value {
        PromptOverride::Custom(content) => view(
            content,
            PromptSource::Custom,
            PromptSelection::Custom,
            native_prompt_available,
        ),
        PromptOverride::Disabled => {
            view("", PromptSource::Custom, PromptSelection::Disabled, native_prompt_available)
        }
    }
}

fn view(
    content: &str,
    source: PromptSource,
    selection: PromptSelection,
    native_prompt_available: Option<bool>,
) -> SystemPromptView {
    SystemPromptView {
        content: content.to_string(),
        source,
        selection,
        disabled: content.is_empty(),
        native_prompt_available,
    }
}
