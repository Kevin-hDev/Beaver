use super::model_customizations;
use super::modelfile_parser::parse_modelfile;
use super::ollama_client::OllamaClient;
use std::sync::OnceLock;

#[cfg(test)]
pub(crate) use store::NativePromptCatalog;
pub(crate) use store::NativePromptStore;
pub use store::NativePromptState;

#[path = "ollama_native_prompt_store.rs"]
mod store;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativePromptOrigin {
    Catalog,
    CurrentModel,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NativePromptLookup {
    Unknown,
    Absent,
    Present(String),
}

impl NativePromptLookup {
    pub fn prompt(&self) -> Option<&str> {
        match self {
            Self::Present(content) => Some(content),
            Self::Unknown | Self::Absent => None,
        }
    }

    pub fn availability(&self) -> Option<bool> {
        match self {
            Self::Unknown => None,
            Self::Absent => Some(false),
            Self::Present(_) => Some(true),
        }
    }
}

pub fn lookup_origin(
    customization: Option<model_customizations::CustomizationKind>,
    cached: bool,
) -> NativePromptOrigin {
    if cached {
        NativePromptOrigin::Catalog
    } else if model_customizations::can_capture_current(customization) {
        NativePromptOrigin::CurrentModel
    } else {
        NativePromptOrigin::Unknown
    }
}

pub async fn get(ollama: &OllamaClient, model: &str) -> NativePromptLookup {
    if model_customizations::validate_model_name(model).is_err() {
        return NativePromptLookup::Unknown;
    }
    let cached_state = cached(model);
    match lookup_origin(
        model_customizations::customization_kind(model),
        cached_state.is_some(),
    ) {
        NativePromptOrigin::Catalog => cached_state
            .as_ref()
            .map(lookup_from_state)
            .unwrap_or(NativePromptLookup::Unknown),
        NativePromptOrigin::Unknown => NativePromptLookup::Unknown,
        NativePromptOrigin::CurrentModel => {
            let Ok(state) = current_state(ollama, model).await else {
                return NativePromptLookup::Unknown;
            };
            let lookup = lookup_from_state(&state);
            let _ = record(model, state);
            lookup
        }
    }
}

pub async fn capture_current(ollama: &OllamaClient, model: &str) -> Result<(), String> {
    let state = current_state(ollama, model).await?;
    record(model, state)
}

pub fn remove(model: &str) -> Result<(), String> {
    model_customizations::validate_model_name(model)?;
    store().remove(model)
}

async fn current_state(ollama: &OllamaClient, model: &str) -> Result<NativePromptState, String> {
    let modelfile = ollama.get_modelfile(model).await?;
    Ok(state_from_modelfile(&modelfile))
}

fn state_from_modelfile(modelfile: &str) -> NativePromptState {
    parse_modelfile(modelfile)
        .system
        .map(|content| content.trim().to_string())
        .filter(|content| !content.is_empty())
        .map(NativePromptState::Present)
        .unwrap_or(NativePromptState::Absent)
}

fn lookup_from_state(state: &NativePromptState) -> NativePromptLookup {
    match state {
        NativePromptState::Absent => NativePromptLookup::Absent,
        NativePromptState::Present(content) => NativePromptLookup::Present(content.clone()),
    }
}

fn cached(model: &str) -> Option<NativePromptState> {
    store().cached(model).ok().flatten()
}

fn record(model: &str, state: NativePromptState) -> Result<(), String> {
    store().record(model, state)
}

fn store() -> &'static NativePromptStore {
    static STORE: OnceLock<NativePromptStore> = OnceLock::new();
    STORE.get_or_init(|| NativePromptStore::open(store_path()))
}

fn store_path() -> std::path::PathBuf {
    crate::services::paths::data_dir().join("ollama-native-system-prompts.json")
}
