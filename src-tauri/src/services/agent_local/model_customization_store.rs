use super::{validate_model_name, CustomizationKind};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

const MAX_CUSTOM_MODELS: usize = 512;
const MAX_STORE_BYTES: u64 = 256 * 1024;
const STORE_ERRORS: crate::services::private_store::StoreErrorCodes =
    crate::services::private_store::StoreErrorCodes::new(
        crate::services::private_store::error_codes::OLLAMA_CUSTOM_MISSING,
        crate::services::private_store::error_codes::OLLAMA_CUSTOM_UNAVAILABLE,
        crate::services::private_store::error_codes::OLLAMA_CUSTOM_WRITE,
    );

#[derive(Clone, Default, Serialize, Deserialize)]
pub(crate) struct ModelCustomizationCatalog {
    models: BTreeMap<String, CustomizationKind>,
}

#[derive(Default, Deserialize)]
struct LegacyCustomModelStore {
    models: Vec<String>,
}

pub(crate) struct ModelCustomizationStore {
    path: PathBuf,
    catalog: Mutex<crate::services::private_store::CachedStore<ModelCustomizationCatalog>>,
}

enum CatalogLoad {
    Missing,
    Ready {
        catalog: ModelCustomizationCatalog,
        migrated: bool,
    },
    Unavailable,
}

impl ModelCustomizationStore {
    pub(crate) fn open(path: PathBuf) -> Self {
        let catalog = crate::services::private_store::CachedStore::new(load_catalog(&path));
        Self {
            path,
            catalog: Mutex::new(catalog),
        }
    }

    pub(crate) fn kind(&self, name: &str) -> Result<Option<CustomizationKind>, String> {
        validate_model_name(name)?;
        let mut current = self
            .catalog
            .lock()
            .map_err(|_| {
                crate::services::private_store::error_codes::OLLAMA_CUSTOM_UNAVAILABLE.to_string()
            })?;
        Ok(current
            .value_or_reload(|| load_catalog(&self.path), &STORE_ERRORS)?
            .kind(name))
    }

    pub(crate) fn mark_parameters(&self, name: &str) -> Result<(), String> {
        self.mutate(|catalog| catalog.mark_parameters(name))
    }

    pub(crate) fn mark_modelfile(&self, name: &str) -> Result<(), String> {
        self.mutate(|catalog| catalog.mark_modelfile(name))
    }

    pub(crate) fn restore_kind(
        &self,
        name: &str,
        kind: Option<CustomizationKind>,
    ) -> Result<(), String> {
        self.mutate(|catalog| catalog.restore_kind(name, kind))
    }

    fn mutate(
        &self,
        update: impl FnOnce(&mut ModelCustomizationCatalog) -> Result<(), String>,
    ) -> Result<(), String> {
        let mut current = self
            .catalog
            .lock()
            .map_err(|_| {
                crate::services::private_store::error_codes::OLLAMA_CUSTOM_WRITE.to_string()
            })?;
        let mut candidate = current.candidate_for_write(
            || load_catalog(&self.path),
            &STORE_ERRORS,
        )?;
        update(&mut candidate)?;
        candidate.write_to_path(&self.path)?;
        current.commit(candidate);
        Ok(())
    }
}

impl ModelCustomizationCatalog {
    pub(crate) fn kind(&self, name: &str) -> Option<CustomizationKind> {
        self.models.get(name).copied()
    }

    pub(crate) fn mark_parameters(&mut self, name: &str) -> Result<(), String> {
        validate_model_name(name)?;
        if self.models.contains_key(name) {
            return Ok(());
        }
        self.insert(name, CustomizationKind::ParametersOnly)
    }

    pub(crate) fn mark_modelfile(&mut self, name: &str) -> Result<(), String> {
        self.insert(name, CustomizationKind::Modelfile)
    }

    pub(crate) fn restore_kind(
        &mut self,
        name: &str,
        kind: Option<CustomizationKind>,
    ) -> Result<(), String> {
        validate_model_name(name)?;
        match kind {
            Some(value) => self.insert(name, value),
            None => {
                self.models.remove(name);
                Ok(())
            }
        }
    }

    #[cfg(test)]
    pub(crate) fn read_from_path(path: &Path) -> Result<Self, String> {
        match load_catalog(path) {
            crate::services::private_store::StoreLoad::Missing => Ok(Self::default()),
            crate::services::private_store::StoreLoad::Ready(catalog) => Ok(catalog),
            crate::services::private_store::StoreLoad::Unavailable(_) => {
                Err(store_unavailable().to_string())
            }
        }
    }

    pub(crate) fn write_to_path(&self, path: &Path) -> Result<(), String> {
        let data = serde_json::to_vec_pretty(self).map_err(|_| {
            crate::services::private_store::error_codes::OLLAMA_CUSTOM_WRITE.to_string()
        })?;
        if data.len() as u64 > MAX_STORE_BYTES {
            return Err("ollama-custom-store-limit".to_string());
        }
        crate::services::private_store::atomic_write(path, &data)
            .map_err(|_| {
                crate::services::private_store::error_codes::OLLAMA_CUSTOM_WRITE.to_string()
            })
    }

    fn read_with_format(path: &Path) -> CatalogLoad {
        let content = match crate::services::private_store::read_bounded_regular(
            path,
            MAX_STORE_BYTES,
        ) {
            Ok(crate::services::private_store::BoundedFile::Missing) => {
                return CatalogLoad::Missing;
            }
            Ok(crate::services::private_store::BoundedFile::Content(content)) => content,
            Err(_) => return CatalogLoad::Unavailable,
        };
        if let Ok(catalog) = serde_json::from_slice::<Self>(&content) {
            return CatalogLoad::Ready {
                catalog: catalog.validated(),
                migrated: false,
            };
        }
        serde_json::from_slice::<LegacyCustomModelStore>(&content)
            .map(|legacy| CatalogLoad::Ready {
                catalog: Self::from_legacy(legacy),
                migrated: true,
            })
            .unwrap_or(CatalogLoad::Unavailable)
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

fn load_catalog(
    path: &Path,
) -> crate::services::private_store::StoreLoad<ModelCustomizationCatalog> {
    match ModelCustomizationCatalog::read_with_format(path) {
        CatalogLoad::Missing => crate::services::private_store::StoreLoad::Missing,
        CatalogLoad::Ready { catalog, migrated } => {
            if migrated && catalog.write_to_path(path).is_err() {
                return crate::services::private_store::StoreLoad::Unavailable(
                    crate::services::private_store::StoreFailure::Write,
                );
            }
            crate::services::private_store::StoreLoad::Ready(catalog)
        }
        CatalogLoad::Unavailable => crate::services::private_store::StoreLoad::Unavailable(
            crate::services::private_store::StoreFailure::Read,
        ),
    }
}

#[cfg(test)]
fn store_unavailable() -> &'static str {
    crate::services::private_store::error_codes::OLLAMA_CUSTOM_UNAVAILABLE
}
