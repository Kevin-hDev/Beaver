use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

const MAX_MODELS: usize = 512;
const MAX_PROMPT_BYTES: usize = 64 * 1024;
const MAX_STORE_BYTES: u64 = 8 * 1024 * 1024;
const STORE_ERRORS: crate::services::private_store::StoreErrorCodes =
    crate::services::private_store::StoreErrorCodes::new(
        "ollama-native-prompt-store-missing",
        "ollama-native-prompt-store-unavailable",
        "ollama-native-prompt-write",
    );

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

pub(crate) struct NativePromptStore {
    path: PathBuf,
    catalog: Mutex<crate::services::private_store::CachedStore<NativePromptCatalog>>,
}

impl NativePromptStore {
    pub(crate) fn open(path: PathBuf) -> Self {
        let catalog = crate::services::private_store::CachedStore::new(
            NativePromptCatalog::load_from_path(&path),
        );
        Self {
            path,
            catalog: Mutex::new(catalog),
        }
    }

    pub(crate) fn cached(&self, model: &str) -> Result<Option<NativePromptState>, String> {
        let mut current = self
            .catalog
            .lock()
            .map_err(|_| "ollama-native-prompt-store-unavailable".to_string())?;
        Ok(current
            .value_or_reload(
                || NativePromptCatalog::load_from_path(&self.path),
                &STORE_ERRORS,
            )?
            .get(model)
            .cloned())
    }

    pub(crate) fn record(&self, model: &str, state: NativePromptState) -> Result<(), String> {
        self.mutate(|catalog| catalog.record(model, state))
    }

    pub(crate) fn remove(&self, model: &str) -> Result<(), String> {
        self.mutate(|catalog| {
            catalog.remove(model);
            Ok(())
        })
    }

    fn mutate(
        &self,
        update: impl FnOnce(&mut NativePromptCatalog) -> Result<(), String>,
    ) -> Result<(), String> {
        let mut current = self
            .catalog
            .lock()
            .map_err(|_| "ollama-native-prompt-store-unavailable".to_string())?;
        let mut candidate = current.candidate_for_write(
            || NativePromptCatalog::load_from_path(&self.path),
            &STORE_ERRORS,
        )?;
        update(&mut candidate)?;
        candidate.write_to_path(&self.path)?;
        current.commit(candidate);
        Ok(())
    }
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

    #[cfg(test)]
    pub fn read_from_path(path: &Path) -> Result<Self, String> {
        match Self::load_from_path(path) {
            crate::services::private_store::StoreLoad::Missing => Ok(Self::default()),
            crate::services::private_store::StoreLoad::Ready(catalog) => Ok(catalog),
            crate::services::private_store::StoreLoad::Unavailable(_) => {
                Err(store_unavailable().to_string())
            }
        }
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

    fn load_from_path(
        path: &Path,
    ) -> crate::services::private_store::StoreLoad<NativePromptCatalog> {
        let content = match crate::services::private_store::read_bounded_regular(
            path,
            MAX_STORE_BYTES,
        ) {
            Ok(crate::services::private_store::BoundedFile::Missing) => {
                return crate::services::private_store::StoreLoad::Missing;
            }
            Ok(crate::services::private_store::BoundedFile::Content(content)) => content,
            Err(_) => {
                return crate::services::private_store::StoreLoad::Unavailable(
                    crate::services::private_store::StoreFailure::Read,
                );
            }
        };
        serde_json::from_slice::<Self>(&content)
            .map(Self::sanitized)
            .map(crate::services::private_store::StoreLoad::Ready)
            .unwrap_or(crate::services::private_store::StoreLoad::Unavailable(
                crate::services::private_store::StoreFailure::Read,
            ))
    }
}

#[cfg(test)]
fn store_unavailable() -> &'static str {
    "ollama-native-prompt-store-unavailable"
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
