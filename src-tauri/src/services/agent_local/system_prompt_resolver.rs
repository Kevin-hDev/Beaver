use super::system_prompt_store::SystemPromptSettings;
use super::ollama_native_prompts::NativePromptLookup;
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

#[cfg(test)]
pub fn resolve_ollama(
    settings: &SystemPromptSettings,
    model: &str,
    mode: PromptMode,
    tier: PromptTier,
    native_prompt: Option<&str>,
    beaver_prompt: &str,
) -> SystemPromptView {
    let native = native_prompt
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| NativePromptLookup::Present(value.to_string()))
        .unwrap_or(NativePromptLookup::Absent);
    resolve_ollama_native(settings, model, mode, tier, &native, beaver_prompt)
}

pub fn resolve_ollama_native(
    settings: &SystemPromptSettings,
    model: &str,
    mode: PromptMode,
    tier: PromptTier,
    native: &NativePromptLookup,
    beaver_prompt: &str,
) -> SystemPromptView {
    let native_available = native.availability();
    if let Some(value) = settings.ollama_override(model, mode, tier) {
        return from_override(value, native_available);
    }
    if settings.ollama_uses_beaver(model, mode, tier) {
        return view(
            beaver_prompt,
            PromptSource::Beaver,
            PromptSelection::Beaver,
            native_available,
        );
    }
    if let Some(content) = native.prompt() {
        return view(content, PromptSource::Ollama, PromptSelection::Default, Some(true));
    }
    match settings.global_override(mode, tier) {
        Some(PromptOverride::Custom(content)) => view(
            content,
            PromptSource::Custom,
            PromptSelection::Default,
            native_available,
        ),
        Some(PromptOverride::Disabled) => {
            view("", PromptSource::Custom, PromptSelection::Default, native_available)
        }
        None => view(
            beaver_prompt,
            PromptSource::Beaver,
            PromptSelection::Default,
            native_available,
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
        native_prompt_available,
    }
}
