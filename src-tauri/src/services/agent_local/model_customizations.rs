use crate::services::agent_local::ollama_client::OllamaClient;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const MAX_CUSTOM_MODELS: usize = 512;
const MAX_MODEL_NAME_LEN: usize = 200;
const MAX_STORE_BYTES: u64 = 64 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CustomizationKind {
    Unknown,
    ParametersOnly,
    Modelfile,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub(crate) struct ModelCustomizationCatalog {
    models: BTreeMap<String, CustomizationKind>,
}

#[derive(Default, Deserialize)]
struct LegacyCustomModelStore {
    models: Vec<String>,
}

impl ModelCustomizationCatalog {
    pub(crate) fn kind(&self, name: &str) -> Option<CustomizationKind> {
        self.models.get(name).copied()
    }

    pub(crate) fn mark_parameters(&mut self, name: &str) -> Result<(), String> {
        self.insert_if_absent(name, CustomizationKind::ParametersOnly)
    }

    pub(crate) fn mark_modelfile(&mut self, name: &str) -> Result<(), String> {
        self.insert(name, CustomizationKind::Modelfile)
    }

    pub(crate) fn read_from_path(path: &Path) -> Self {
        if std::fs::metadata(path)
            .map(|metadata| metadata.len() > MAX_STORE_BYTES)
            .unwrap_or(false)
        {
            return Self::default();
        }
        let Ok(content) = std::fs::read_to_string(path) else {
            return Self::default();
        };
        if let Ok(catalog) = serde_json::from_str::<Self>(&content) {
            return catalog.validated();
        }
        serde_json::from_str::<LegacyCustomModelStore>(&content)
            .map(Self::from_legacy)
            .unwrap_or_default()
    }

    pub(crate) fn write_to_path(&self, path: &Path) -> Result<(), String> {
        let parent = path
            .parent()
            .ok_or_else(|| "ollama-custom-store-path".to_string())?;
        std::fs::create_dir_all(parent).map_err(|_| "ollama-custom-store-write".to_string())?;
        let data =
            serde_json::to_vec_pretty(self).map_err(|_| "ollama-custom-store-write".to_string())?;
        let temporary = path.with_extension("tmp");
        std::fs::write(&temporary, data).map_err(|_| "ollama-custom-store-write".to_string())?;
        std::fs::rename(&temporary, path).map_err(|_| "ollama-custom-store-write".to_string())
    }

    fn insert_if_absent(
        &mut self,
        name: &str,
        kind: CustomizationKind,
    ) -> Result<(), String> {
        validate_model_name(name)?;
        if self.models.contains_key(name) {
            return Ok(());
        }
        self.insert(name, kind)
    }

    fn insert(&mut self, name: &str, kind: CustomizationKind) -> Result<(), String> {
        validate_model_name(name)?;
        if self.models.len() >= MAX_CUSTOM_MODELS && !self.models.contains_key(name) {
            return Err("ollama-custom-model-limit".into());
        }
        self.models.insert(name.to_string(), kind);
        Ok(())
    }

    fn validated(mut self) -> Self {
        self.models
            .retain(|name, _| validate_model_name(name).is_ok());
        self.models = self.models.into_iter().take(MAX_CUSTOM_MODELS).collect();
        self
    }

    fn from_legacy(store: LegacyCustomModelStore) -> Self {
        Self {
            models: store
                .models
                .into_iter()
                .filter(|name| validate_model_name(name).is_ok())
                .take(MAX_CUSTOM_MODELS)
                .map(|name| (name, CustomizationKind::Unknown))
                .collect(),
        }
    }
}

pub fn customization_kind(name: &str) -> Option<CustomizationKind> {
    ModelCustomizationCatalog::read_from_path(&store_path()).kind(name)
}

pub fn is_model_customized(name: &str) -> bool {
    customization_kind(name).is_some()
}

pub fn can_capture_current(kind: Option<CustomizationKind>) -> bool {
    matches!(kind, None | Some(CustomizationKind::ParametersOnly))
}

pub fn mark_parameters_customized(name: &str) -> Result<(), String> {
    mutate_catalog(|catalog| catalog.mark_parameters(name))
}

pub fn mark_modelfile_customized(name: &str) -> Result<(), String> {
    mutate_catalog(|catalog| catalog.mark_modelfile(name))
}

pub fn restore_customization_kind(
    name: &str,
    kind: Option<CustomizationKind>,
) -> Result<(), String> {
    mutate_catalog(|catalog| {
        validate_model_name(name)?;
        match kind {
            Some(value) => catalog.insert(name, value),
            None => {
                catalog.models.remove(name);
                Ok(())
            }
        }
    })
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

fn mutate_catalog(
    update: impl FnOnce(&mut ModelCustomizationCatalog) -> Result<(), String>,
) -> Result<(), String> {
    let path = store_path();
    let mut catalog = ModelCustomizationCatalog::read_from_path(&path);
    update(&mut catalog)?;
    catalog.write_to_path(&path)
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
