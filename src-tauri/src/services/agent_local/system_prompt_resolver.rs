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
        Some(value) => from_override(value, beaver_prompt),
        None => view(beaver_prompt, PromptSource::Beaver, PromptSelection::Default),
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
    if let Some(explicit) =
        resolve_ollama_without_native(settings, model, mode, tier, beaver_prompt)
    {
        return explicit;
    }
    if let Some(native) = native_prompt.map(str::trim).filter(|value| !value.is_empty()) {
        return view(native, PromptSource::Ollama, PromptSelection::Default);
    }
    match settings.global_override(mode, tier) {
        Some(PromptOverride::Custom(content)) => view(
            content,
            PromptSource::Custom,
            PromptSelection::Default,
        ),
        Some(PromptOverride::Disabled) => {
            view("", PromptSource::Custom, PromptSelection::Default)
        }
        Some(PromptOverride::Beaver) | None => view(
            beaver_prompt,
            PromptSource::Beaver,
            PromptSelection::Default,
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
        return Some(from_override(value, beaver_prompt));
    }
    settings.ollama_uses_beaver(model, mode, tier).then(|| {
        view(
            beaver_prompt,
            PromptSource::Beaver,
            PromptSelection::Beaver,
        )
    })
}

fn from_override(value: &PromptOverride, beaver_prompt: &str) -> SystemPromptView {
    match value {
        PromptOverride::Custom(content) => view(
            content,
            PromptSource::Custom,
            PromptSelection::Custom,
        ),
        PromptOverride::Disabled => {
            view("", PromptSource::Custom, PromptSelection::Disabled)
        }
        PromptOverride::Beaver => view(
            beaver_prompt,
            PromptSource::Beaver,
            PromptSelection::Beaver,
        ),
    }
}

fn view(
    content: &str,
    source: PromptSource,
    selection: PromptSelection,
) -> SystemPromptView {
    SystemPromptView {
        content: content.to_string(),
        source,
        selection,
        disabled: content.is_empty(),
    }
}
