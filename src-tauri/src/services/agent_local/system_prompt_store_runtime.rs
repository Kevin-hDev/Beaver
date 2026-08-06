use super::SystemPromptSettings;
use crate::services::agent_local::system_prompt_types::{PromptMode, PromptTier};
use std::sync::{Mutex, OnceLock};

pub fn snapshot() -> Result<SystemPromptSettings, String> {
    store_cache()
        .lock()
        .map(|settings| settings.clone())
        .map_err(|_| "system-prompt-store-read".to_string())
}

pub fn save_global(mode: PromptMode, tier: PromptTier, prompt: &str) -> Result<(), String> {
    mutate(|settings| settings.set_global(mode, tier, prompt))
}

pub fn save_ollama(
    model: &str,
    mode: PromptMode,
    tier: PromptTier,
    prompt: &str,
) -> Result<(), String> {
    mutate(|settings| settings.set_ollama(model, mode, tier, prompt))
}

pub fn restore_global(mode: PromptMode, tier: PromptTier) -> Result<(), String> {
    mutate(|settings| {
        settings.restore_global(mode, tier);
        Ok(())
    })
}

pub fn restore_ollama(model: &str, mode: PromptMode, tier: PromptTier) -> Result<(), String> {
    mutate(|settings| settings.select_ollama_beaver(model, mode, tier))
}

pub fn restore_ollama_default(
    model: &str,
    mode: PromptMode,
    tier: PromptTier,
) -> Result<(), String> {
    mutate(|settings| settings.restore_ollama_default(model, mode, tier))
}

pub fn remove_ollama_model(model: &str) -> Result<(), String> {
    super::super::model_customizations::validate_model_name(model)?;
    mutate(|settings| {
        settings.remove_ollama_model(model);
        Ok(())
    })
}

fn mutate(
    update: impl FnOnce(&mut SystemPromptSettings) -> Result<(), String>,
) -> Result<(), String> {
    let mut current = store_cache()
        .lock()
        .map_err(|_| "system-prompt-store-write".to_string())?;
    let mut candidate = current.clone();
    update(&mut candidate)?;
    candidate.write_to_path(&store_path())?;
    *current = candidate;
    Ok(())
}

fn store_cache() -> &'static Mutex<SystemPromptSettings> {
    static STORE: OnceLock<Mutex<SystemPromptSettings>> = OnceLock::new();
    STORE.get_or_init(|| {
        Mutex::new(SystemPromptSettings::read_with_legacy(
            &store_path(),
            &legacy_store_path(),
        ))
    })
}

fn store_path() -> std::path::PathBuf {
    crate::services::paths::data_dir().join("system-prompt-settings.json")
}

fn legacy_store_path() -> std::path::PathBuf {
    crate::services::paths::data_dir().join("ollama-system-prompts.json")
}
