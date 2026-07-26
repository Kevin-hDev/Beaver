use crate::services::agent_local::ollama_client::OllamaClient;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::PathBuf;

const MAX_CUSTOM_MODELS: usize = 512;
const MAX_MODEL_NAME_LEN: usize = 200;
const MAX_STORE_BYTES: u64 = 64 * 1024;

#[derive(Default, Serialize, Deserialize)]
struct CustomModelStore {
    models: Vec<String>,
}

pub fn is_model_customized(name: &str) -> bool {
    read_model_names().contains(name)
}

pub fn mark_model_customized(name: &str) -> Result<(), String> {
    validate_model_name(name)?;
    let mut names = read_model_names();
    if names.len() >= MAX_CUSTOM_MODELS && !names.contains(name) {
        return Err("ollama-custom-model-limit".into());
    }
    names.insert(name.to_string());
    write_model_names(&names)
}

pub fn clear_model_customized(name: &str) -> Result<(), String> {
    validate_model_name(name)?;
    let mut names = read_model_names();
    names.remove(name);
    write_model_names(&names)
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

fn read_model_names() -> BTreeSet<String> {
    let path = store_path();
    if std::fs::metadata(&path)
        .map(|m| m.len() > MAX_STORE_BYTES)
        .unwrap_or(false)
    {
        return BTreeSet::new();
    }
    let Ok(content) = std::fs::read_to_string(path) else {
        return BTreeSet::new();
    };
    serde_json::from_str::<CustomModelStore>(&content)
        .map(|store| {
            store
                .models
                .into_iter()
                .filter(|name| validate_model_name(name).is_ok())
                .take(MAX_CUSTOM_MODELS)
                .collect()
        })
        .unwrap_or_default()
}

fn write_model_names(names: &BTreeSet<String>) -> Result<(), String> {
    let path = store_path();
    let parent = path
        .parent()
        .ok_or_else(|| "ollama-custom-store-path".to_string())?;
    std::fs::create_dir_all(parent).map_err(|_| "ollama-custom-store-write".to_string())?;
    let store = CustomModelStore {
        models: names.iter().take(MAX_CUSTOM_MODELS).cloned().collect(),
    };
    let data =
        serde_json::to_vec_pretty(&store).map_err(|_| "ollama-custom-store-write".to_string())?;
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, data).map_err(|_| "ollama-custom-store-write".to_string())?;
    std::fs::rename(&tmp, &path).map_err(|_| "ollama-custom-store-write".to_string())
}

fn store_path() -> PathBuf {
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
