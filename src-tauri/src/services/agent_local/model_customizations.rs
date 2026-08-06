use crate::services::agent_local::ollama_client::OllamaClient;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

#[cfg(test)]
pub(crate) use store::ModelCustomizationCatalog;
pub(crate) use store::ModelCustomizationStore;

#[path = "model_customization_store.rs"]
mod store;

const MAX_MODEL_NAME_LEN: usize = 200;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CustomizationKind {
    Unknown,
    ParametersOnly,
    Modelfile,
}

pub fn customization_kind(name: &str) -> Option<CustomizationKind> {
    customization_kind_from(runtime_store(), name)
}

pub(crate) fn customization_kind_from(
    store: &ModelCustomizationStore,
    name: &str,
) -> Option<CustomizationKind> {
    if validate_model_name(name).is_err() {
        return None;
    }
    store.kind(name).unwrap_or(Some(CustomizationKind::Unknown))
}

pub fn is_model_customized(name: &str) -> bool {
    customization_kind(name).is_some()
}

pub fn can_capture_current(kind: Option<CustomizationKind>) -> bool {
    matches!(kind, None | Some(CustomizationKind::ParametersOnly))
}

pub fn mark_parameters_customized(name: &str) -> Result<(), String> {
    runtime_store().mark_parameters(name)
}

pub fn mark_modelfile_customized(name: &str) -> Result<(), String> {
    runtime_store().mark_modelfile(name)
}

pub fn restore_customization_kind(
    name: &str,
    kind: Option<CustomizationKind>,
) -> Result<(), String> {
    runtime_store().restore_kind(name, kind)
}

pub fn clear_model_customized(name: &str) -> Result<(), String> {
    restore_customization_kind(name, None)
}

pub async fn save_for_update(ollama: &OllamaClient, name: &str) -> Option<String> {
    if !is_model_customized(name) {
        return None;
    }
    ollama.get_modelfile(name).await.ok()
}

pub async fn restore_after_update(ollama: &OllamaClient, name: &str, saved: &str) {
    let restored = super::ollama_modelfile_create::use_updated_base(saved, name);
    if let Err(e) = ollama.update_modelfile(name, &restored).await {
        eprintln!("[pull] restore perso {name} échoué: {e}");
    }
}

fn runtime_store() -> &'static ModelCustomizationStore {
    static STORE: OnceLock<ModelCustomizationStore> = OnceLock::new();
    STORE.get_or_init(|| ModelCustomizationStore::open(store_path()))
}

fn store_path() -> std::path::PathBuf {
    crate::services::paths::data_dir().join("ollama-custom-models.json")
}

pub(crate) fn validate_model_name(name: &str) -> Result<(), String> {
    if name.is_empty() || name.len() > MAX_MODEL_NAME_LEN {
        return Err("ollama-model-name-invalid".into());
    }
    if name.contains("..") || !name.chars().all(is_allowed_model_char) {
        return Err("ollama-model-name-invalid".into());
    }
    Ok(())
}

fn is_allowed_model_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | ':' | '/')
}
