use super::{PromptMatrix, SystemPromptSettings, MAX_MODELS, MAX_PROMPT_BYTES};
use crate::services::agent_local::system_prompt_types::{
    PromptMode, PromptOverride, PromptTier,
};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;

const MAX_STORE_BYTES: u64 = 8 * 1024 * 1024;

#[derive(Deserialize)]
struct LegacyStore {
    prompts: BTreeMap<String, String>,
}

impl SystemPromptSettings {
    pub fn read_from_path(path: &Path) -> Self {
        read_bounded(path)
            .and_then(|content| serde_json::from_str::<Self>(&content).ok())
            .map(Self::sanitized)
            .unwrap_or_default()
    }

    pub fn read_with_legacy(path: &Path, legacy_path: &Path) -> Self {
        if path.exists() {
            return Self::read_from_path(path);
        }
        let migrated = migrate_legacy(legacy_path);
        if !migrated.is_empty() {
            let _ = migrated.write_to_path(path);
        }
        migrated
    }

    pub fn write_to_path(&self, path: &Path) -> Result<(), String> {
        let data = serde_json::to_vec_pretty(self)
            .map_err(|_| "system-prompt-store-write".to_string())?;
        if data.len() as u64 > MAX_STORE_BYTES {
            return Err("system-prompt-store-limit".into());
        }
        crate::services::private_store::atomic_write(path, &data)
            .map_err(|_| "system-prompt-store-write".to_string())
    }

    fn sanitized(self) -> Self {
        let mut clean = Self::default();
        copy_matrix(&mut clean.global, &self.global);
        for (model, matrix) in self.ollama.into_iter().take(MAX_MODELS) {
            if super::super::model_customizations::validate_model_name(&model).is_err() {
                continue;
            }
            let mut target = PromptMatrix::default();
            copy_matrix(&mut target, &matrix);
            if !target.is_empty() {
                clean.ollama.insert(model, target);
            }
        }
        clean
    }

    fn is_empty(&self) -> bool {
        self.global.is_empty() && self.ollama.is_empty()
    }
}

fn migrate_legacy(path: &Path) -> SystemPromptSettings {
    let Some(content) = read_bounded(path) else {
        return SystemPromptSettings::default();
    };
    let Ok(legacy) = serde_json::from_str::<LegacyStore>(&content) else {
        return SystemPromptSettings::default();
    };
    let mut settings = SystemPromptSettings::default();
    for (model, prompt) in legacy.prompts.into_iter().take(MAX_MODELS) {
        for mode in [PromptMode::Chatbot, PromptMode::Agentic] {
            for tier in [PromptTier::Compact, PromptTier::Detailed] {
                let _ = settings.set_ollama(&model, mode, tier, &prompt);
            }
        }
    }
    settings
}

fn copy_matrix(target: &mut PromptMatrix, source: &PromptMatrix) {
    for mode in [PromptMode::Chatbot, PromptMode::Agentic] {
        for tier in [PromptTier::Compact, PromptTier::Detailed] {
            if source.beaver(mode, tier)
                || matches!(source.get(mode, tier), Some(PromptOverride::Beaver))
            {
                target.set_beaver(mode, tier, true);
                continue;
            }
            let Some(value) = source.get(mode, tier).and_then(sanitize_override) else {
                continue;
            };
            *target.get_mut(mode, tier) = Some(value);
        }
    }
}

fn sanitize_override(value: &PromptOverride) -> Option<PromptOverride> {
    match value {
        PromptOverride::Disabled => Some(PromptOverride::Disabled),
        PromptOverride::Beaver => None,
        PromptOverride::Custom(content)
            if !content.contains('\0') && content.len() <= MAX_PROMPT_BYTES =>
        {
            let trimmed = content.trim();
            (!trimmed.is_empty()).then(|| PromptOverride::Custom(trimmed.to_string()))
        }
        PromptOverride::Custom(_) => None,
    }
}

fn read_bounded(path: &Path) -> Option<String> {
    if std::fs::metadata(path).ok()?.len() > MAX_STORE_BYTES {
        return None;
    }
    std::fs::read_to_string(path).ok()
}
