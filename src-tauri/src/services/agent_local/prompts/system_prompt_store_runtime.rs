use super::SystemPromptSettings;
use crate::services::agent_local::system_prompt_types::{PromptMode, PromptTier};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

const STORE_ERRORS: crate::services::private_store::StoreErrorCodes =
    crate::services::private_store::StoreErrorCodes::new(
        crate::services::private_store::error_codes::SYSTEM_PROMPT_MISSING,
        crate::services::private_store::error_codes::SYSTEM_PROMPT_UNAVAILABLE,
        crate::services::private_store::error_codes::SYSTEM_PROMPT_WRITE,
    );

pub fn snapshot() -> Result<SystemPromptSettings, String> {
    runtime_store().snapshot()
}

pub struct RuntimeSystemPromptSettings {
    pub settings: SystemPromptSettings,
    pub notice_key: Option<&'static str>,
}

pub fn snapshot_for_runtime() -> RuntimeSystemPromptSettings {
    runtime_store().snapshot_for_runtime()
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
                &STORE_ERRORS,
            )
            .cloned()
    }

    pub(crate) fn snapshot_for_runtime(&self) -> RuntimeSystemPromptSettings {
        match self.snapshot() {
            Ok(settings) => RuntimeSystemPromptSettings {
                settings,
                notice_key: None,
            },
            Err(_) => RuntimeSystemPromptSettings {
                settings: SystemPromptSettings::default(),
                notice_key: Some("errors.localStore.systemPromptsRuntimeFallback"),
            },
        }
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
            &STORE_ERRORS,
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
    crate::services::private_store::error_codes::SYSTEM_PROMPT_UNAVAILABLE
}
