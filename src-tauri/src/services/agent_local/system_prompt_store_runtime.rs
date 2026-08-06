use super::SystemPromptSettings;
use crate::services::agent_local::system_prompt_types::{PromptMode, PromptTier};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

pub fn snapshot() -> Result<SystemPromptSettings, String> {
    runtime_store().snapshot()
}

pub fn save_global(mode: PromptMode, tier: PromptTier, prompt: &str) -> Result<(), String> {
    runtime_store().save_global(mode, tier, prompt)
}

pub fn save_ollama(
    model: &str,
    mode: PromptMode,
    tier: PromptTier,
    prompt: &str,
) -> Result<(), String> {
    runtime_store().mutate(|settings| settings.set_ollama(model, mode, tier, prompt))
}

pub fn restore_global(mode: PromptMode, tier: PromptTier) -> Result<(), String> {
    runtime_store().mutate(|settings| {
        settings.restore_global(mode, tier);
        Ok(())
    })
}

pub fn restore_ollama(model: &str, mode: PromptMode, tier: PromptTier) -> Result<(), String> {
    runtime_store().mutate(|settings| settings.select_ollama_beaver(model, mode, tier))
}

pub fn restore_ollama_default(
    model: &str,
    mode: PromptMode,
    tier: PromptTier,
) -> Result<(), String> {
    runtime_store().mutate(|settings| settings.restore_ollama_default(model, mode, tier))
}

pub fn remove_ollama_model(model: &str) -> Result<(), String> {
    super::super::model_customizations::validate_model_name(model)?;
    runtime_store().mutate(|settings| {
        settings.remove_ollama_model(model);
        Ok(())
    })
}

pub(crate) struct SystemPromptSettingsStore {
    path: PathBuf,
    settings: Mutex<crate::services::private_store::CachedStore<SystemPromptSettings>>,
}

impl SystemPromptSettingsStore {
    pub(crate) fn open(path: PathBuf, legacy_path: PathBuf) -> Self {
        let settings = crate::services::private_store::CachedStore::new(
            SystemPromptSettings::load_with_legacy(&path, &legacy_path),
        );
        Self {
            path,
            settings: Mutex::new(settings),
        }
    }

    pub(crate) fn snapshot(&self) -> Result<SystemPromptSettings, String> {
        let mut current = self
            .settings
            .lock()
            .map_err(|_| store_unavailable())?;
        current
            .value_or_reload(
                || SystemPromptSettings::load_from_path(&self.path),
                store_unavailable(),
            )
            .cloned()
    }

    pub(crate) fn save_global(
        &self,
        mode: PromptMode,
        tier: PromptTier,
        prompt: &str,
    ) -> Result<(), String> {
        self.mutate(|settings| settings.set_global(mode, tier, prompt))
    }

    fn mutate(
        &self,
        update: impl FnOnce(&mut SystemPromptSettings) -> Result<(), String>,
    ) -> Result<(), String> {
        let mut current = self
            .settings
            .lock()
            .map_err(|_| store_unavailable())?;
        let mut candidate = current.candidate_for_write(
            || SystemPromptSettings::load_from_path(&self.path),
            store_unavailable(),
        )?;
        update(&mut candidate)?;
        candidate.write_to_path(&self.path)?;
        current.commit(candidate);
        Ok(())
    }
}

fn runtime_store() -> &'static SystemPromptSettingsStore {
    static STORE: OnceLock<SystemPromptSettingsStore> = OnceLock::new();
    STORE.get_or_init(|| SystemPromptSettingsStore::open(store_path(), legacy_store_path()))
}

fn store_path() -> std::path::PathBuf {
    crate::services::paths::data_dir().join("system-prompt-settings.json")
}

fn legacy_store_path() -> std::path::PathBuf {
    crate::services::paths::data_dir().join("ollama-system-prompts.json")
}

fn store_unavailable() -> &'static str {
    "system-prompt-store-unavailable"
}
