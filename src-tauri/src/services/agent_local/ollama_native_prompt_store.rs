use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

const MAX_MODELS: usize = 512;
const MAX_PROMPT_BYTES: usize = 64 * 1024;
const MAX_STORE_BYTES: u64 = 8 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", content = "content", rename_all = "lowercase")]
pub enum NativePromptState {
    Absent,
    Present(String),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NativePromptCatalog {
    models: BTreeMap<String, NativePromptState>,
}

impl NativePromptCatalog {
    pub fn get(&self, model: &str) -> Option<&NativePromptState> {
        self.models.get(model)
    }

    pub fn record(&mut self, model: &str, state: NativePromptState) -> Result<(), String> {
        super::super::model_customizations::validate_model_name(model)?;
        if self.models.len() >= MAX_MODELS && !self.models.contains_key(model) {
            return Err("ollama-native-prompt-limit".into());
        }
        let state = sanitize_state(state).ok_or_else(|| "ollama-native-prompt-invalid".to_string())?;
        self.models.insert(model.to_string(), state);
        Ok(())
    }

    pub fn remove(&mut self, model: &str) {
        self.models.remove(model);
    }

    pub fn read_from_path(path: &Path) -> Self {
        if std::fs::metadata(path)
            .map(|metadata| metadata.len() > MAX_STORE_BYTES)
            .unwrap_or(false)
        {
            return Self::default();
        }
        let Ok(content) = std::fs::read_to_string(path) else {
            return Self::default();
        };
        let Ok(catalog) = serde_json::from_str::<Self>(&content) else {
            return Self::default();
        };
        catalog.sanitized()
    }

    pub fn write_to_path(&self, path: &Path) -> Result<(), String> {
        let data = serde_json::to_vec_pretty(self)
            .map_err(|_| "ollama-native-prompt-write".to_string())?;
        if data.len() as u64 > MAX_STORE_BYTES {
            return Err("ollama-native-prompt-limit".into());
        }
        crate::services::private_store::atomic_write(path, &data)
            .map_err(|_| "ollama-native-prompt-write".to_string())
    }

    fn sanitized(self) -> Self {
        let mut clean = Self::default();
        for (model, state) in self.models.into_iter().take(MAX_MODELS) {
            if super::super::model_customizations::validate_model_name(&model).is_err() {
                continue;
            }
            if let Some(state) = sanitize_state(state) {
                clean.models.insert(model, state);
            }
        }
        clean
    }
}

fn sanitize_state(state: NativePromptState) -> Option<NativePromptState> {
    match state {
        NativePromptState::Absent => Some(NativePromptState::Absent),
        NativePromptState::Present(content)
            if content.len() <= MAX_PROMPT_BYTES && !content.contains('\0') =>
        {
            let trimmed = content.trim();
            if trimmed.is_empty() {
                Some(NativePromptState::Absent)
            } else {
                Some(NativePromptState::Present(trimmed.to_string()))
            }
        }
        NativePromptState::Present(_) => None,
    }
}
