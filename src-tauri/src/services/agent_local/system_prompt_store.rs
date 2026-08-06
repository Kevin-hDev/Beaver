use super::system_prompt_types::{PromptMode, PromptOverride, PromptTier};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};

const MAX_MODELS: usize = 512;
const MAX_PROMPT_BYTES: usize = 64 * 1024;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemPromptSettings {
    global: PromptMatrix,
    ollama: BTreeMap<String, PromptMatrix>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct PromptMatrix {
    chatbot: PromptPair,
    agentic: PromptPair,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct PromptPair {
    compact: Option<PromptOverride>,
    detailed: Option<PromptOverride>,
    #[serde(default)]
    compact_beaver: bool,
    #[serde(default)]
    detailed_beaver: bool,
}

impl SystemPromptSettings {
    pub fn global_override(
        &self,
        mode: PromptMode,
        tier: PromptTier,
    ) -> Option<&PromptOverride> {
        self.global.get(mode, tier)
    }

    pub fn ollama_override(
        &self,
        model: &str,
        mode: PromptMode,
        tier: PromptTier,
    ) -> Option<&PromptOverride> {
        self.ollama.get(model)?.get(mode, tier)
    }

    pub fn set_global(
        &mut self,
        mode: PromptMode,
        tier: PromptTier,
        prompt: &str,
    ) -> Result<(), String> {
        *self.global.get_mut(mode, tier) = Some(normalize_override(prompt)?);
        Ok(())
    }

    pub fn set_ollama(
        &mut self,
        model: &str,
        mode: PromptMode,
        tier: PromptTier,
        prompt: &str,
    ) -> Result<(), String> {
        super::model_customizations::validate_model_name(model)?;
        self.ensure_model_capacity(model)?;
        let matrix = self.ollama.entry(model.to_string()).or_default();
        *matrix.get_mut(mode, tier) = Some(normalize_override(prompt)?);
        matrix.set_beaver(mode, tier, false);
        Ok(())
    }

    pub fn restore_global(&mut self, mode: PromptMode, tier: PromptTier) {
        *self.global.get_mut(mode, tier) = None;
    }

    pub fn remove_ollama_model(&mut self, model: &str) {
        self.ollama.remove(model);
    }
}

impl PromptMatrix {
    fn get(&self, mode: PromptMode, tier: PromptTier) -> Option<&PromptOverride> {
        self.pair(mode).get(tier)
    }

    fn get_mut(&mut self, mode: PromptMode, tier: PromptTier) -> &mut Option<PromptOverride> {
        self.pair_mut(mode).get_mut(tier)
    }

    fn pair(&self, mode: PromptMode) -> &PromptPair {
        match mode {
            PromptMode::Chatbot => &self.chatbot,
            PromptMode::Agentic => &self.agentic,
        }
    }

    fn pair_mut(&mut self, mode: PromptMode) -> &mut PromptPair {
        match mode {
            PromptMode::Chatbot => &mut self.chatbot,
            PromptMode::Agentic => &mut self.agentic,
        }
    }

    fn is_empty(&self) -> bool {
        self.chatbot.is_empty() && self.agentic.is_empty()
    }
}

impl PromptPair {
    fn get(&self, tier: PromptTier) -> Option<&PromptOverride> {
        match tier {
            PromptTier::Compact => self.compact.as_ref(),
            PromptTier::Detailed => self.detailed.as_ref(),
        }
    }

    fn get_mut(&mut self, tier: PromptTier) -> &mut Option<PromptOverride> {
        match tier {
            PromptTier::Compact => &mut self.compact,
            PromptTier::Detailed => &mut self.detailed,
        }
    }

    fn is_empty(&self) -> bool {
        self.compact.is_none()
            && self.detailed.is_none()
            && !self.compact_beaver
            && !self.detailed_beaver
    }
}

fn normalize_override(prompt: &str) -> Result<PromptOverride, String> {
    let normalized = prompt.trim();
    if normalized.len() > MAX_PROMPT_BYTES || normalized.contains('\0') {
        return Err("system-prompt-invalid".into());
    }
    if normalized.is_empty() {
        Ok(PromptOverride::Disabled)
    } else {
        Ok(PromptOverride::Custom(normalized.to_string()))
    }
}

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
    super::model_customizations::validate_model_name(model)?;
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

#[path = "system_prompt_persistence.rs"]
mod persistence;

#[path = "system_prompt_store_selection.rs"]
mod selection;
