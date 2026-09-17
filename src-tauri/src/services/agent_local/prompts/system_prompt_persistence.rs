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

pub(super) type SettingsLoad =
    crate::services::private_store::StoreLoad<SystemPromptSettings>;

impl SystemPromptSettings {
    #[cfg(test)]
    pub fn read_from_path(path: &Path) -> Result<Self, String> {
        match Self::load_from_path(path) {
            SettingsLoad::Missing => Ok(Self::default()),
            SettingsLoad::Ready(settings) => Ok(settings),
            SettingsLoad::Unavailable(failure) => Err(store_failure(failure)),
        }
    }

    #[cfg(test)]
    pub fn read_with_legacy(path: &Path, legacy_path: &Path) -> Result<Self, String> {
        match Self::load_with_legacy(path, legacy_path) {
            SettingsLoad::Missing => Ok(Self::default()),
            SettingsLoad::Ready(settings) => Ok(settings),
            SettingsLoad::Unavailable(failure) => Err(store_failure(failure)),
        }
    }

    pub fn write_to_path(&self, path: &Path) -> Result<(), String> {
        let data = serde_json::to_vec_pretty(self).map_err(|_| {
            crate::services::private_store::error_codes::SYSTEM_PROMPT_WRITE.to_string()
        })?;
        if data.len() as u64 > MAX_STORE_BYTES {
            return Err("system-prompt-store-limit".into());
        }
        crate::services::private_store::atomic_write(path, &data)
            .map_err(|_| {
                crate::services::private_store::error_codes::SYSTEM_PROMPT_WRITE.to_string()
            })
    }

    fn sanitized(self) -> Self {
        let mut clean = Self::default();
        copy_matrix(&mut clean.global, &self.global, false);
        for (model, matrix) in self.ollama.into_iter().take(MAX_MODELS) {
            if super::super::model_customizations::validate_model_name(&model).is_err() {
                continue;
            }
            let mut target = PromptMatrix::default();
            copy_matrix(&mut target, &matrix, true);
            if !target.is_empty() {
                clean.ollama.insert(model, target);
            }
        }
        clean
    }

    pub(super) fn load_with_legacy(path: &Path, legacy_path: &Path) -> SettingsLoad {
        match Self::load_from_path(path) {
            SettingsLoad::Ready(settings) => SettingsLoad::Ready(settings),
            SettingsLoad::Unavailable(failure) => SettingsLoad::Unavailable(failure),
            SettingsLoad::Missing => match migration_archive_exists(legacy_path) {
                Err(()) => SettingsLoad::Unavailable(
                    crate::services::private_store::StoreFailure::Read,
                ),
                Ok(true) => SettingsLoad::Missing,
                Ok(false) => match migrate_legacy(legacy_path) {
                SettingsLoad::Ready(settings) => {
                    if settings.write_to_path(path).is_err() {
                        return SettingsLoad::Unavailable(
                            crate::services::private_store::StoreFailure::Write,
                        );
                    }
                    // The new settings are already durable. Archiving the source is
                    // best-effort so a backup problem cannot disable valid settings.
                    let _ = archive_legacy(legacy_path);
                    SettingsLoad::Ready(settings)
                }
                other => other,
                },
            },
        }
    }

    pub(super) fn load_from_path(path: &Path) -> SettingsLoad {
        let content = match crate::services::private_store::read_bounded_regular(
            path,
            MAX_STORE_BYTES,
        ) {
            Ok(crate::services::private_store::BoundedFile::Missing) => {
                return SettingsLoad::Missing;
            }
            Ok(crate::services::private_store::BoundedFile::Content(content)) => content,
            Err(_) => {
                return SettingsLoad::Unavailable(
                    crate::services::private_store::StoreFailure::Read,
                );
            }
        };
        serde_json::from_slice::<Self>(&content)
            .map(Self::sanitized)
            .map(SettingsLoad::Ready)
            .unwrap_or(SettingsLoad::Unavailable(
                crate::services::private_store::StoreFailure::Read,
            ))
    }
}

fn migrate_legacy(path: &Path) -> SettingsLoad {
    let content = match crate::services::private_store::read_bounded_regular(path, MAX_STORE_BYTES) {
        Ok(crate::services::private_store::BoundedFile::Missing) => {
            return SettingsLoad::Missing;
        }
        Ok(crate::services::private_store::BoundedFile::Content(content)) => content,
        Err(_) => {
            return SettingsLoad::Unavailable(
                crate::services::private_store::StoreFailure::Read,
            );
        }
    };
    let Ok(legacy) = serde_json::from_slice::<LegacyStore>(&content) else {
        return SettingsLoad::Unavailable(
            crate::services::private_store::StoreFailure::Read,
        );
    };
    let mut settings = SystemPromptSettings::default();
    for (model, prompt) in legacy.prompts.into_iter().take(MAX_MODELS) {
        for mode in [PromptMode::Chatbot, PromptMode::Agentic] {
            for tier in [PromptTier::Compact, PromptTier::Detailed] {
                let _ = settings.set_ollama(&model, mode, tier, &prompt);
            }
        }
    }
    SettingsLoad::Ready(settings)
}

fn copy_matrix(target: &mut PromptMatrix, source: &PromptMatrix, keep_beaver: bool) {
    for mode in [PromptMode::Chatbot, PromptMode::Agentic] {
        for tier in [PromptTier::Compact, PromptTier::Detailed] {
            if keep_beaver && source.beaver(mode, tier) {
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
        PromptOverride::Custom(content)
            if !content.contains('\0') && content.len() <= MAX_PROMPT_BYTES =>
        {
            let trimmed = content.trim();
            (!trimmed.is_empty()).then(|| PromptOverride::Custom(trimmed.to_string()))
        }
        PromptOverride::Custom(_) => None,
    }
}

#[cfg(test)]
fn store_unavailable() -> String {
    crate::services::private_store::error_codes::SYSTEM_PROMPT_UNAVAILABLE.to_string()
}

#[cfg(test)]
fn store_failure(failure: crate::services::private_store::StoreFailure) -> String {
    match failure {
        crate::services::private_store::StoreFailure::Read => store_unavailable(),
        crate::services::private_store::StoreFailure::Write => {
            crate::services::private_store::error_codes::SYSTEM_PROMPT_WRITE.to_string()
        }
    }
}

fn migration_archive_path(path: &Path) -> std::path::PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(".migrated");
    std::path::PathBuf::from(name)
}

fn migration_archive_exists(path: &Path) -> Result<bool, ()> {
    match std::fs::symlink_metadata(migration_archive_path(path)) {
        Ok(metadata) if metadata.is_file() => Ok(true),
        Ok(_) => Err(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(_) => Err(()),
    }
}

fn archive_legacy(path: &Path) -> Result<(), String> {
    let archive = migration_archive_path(path);
    if migration_archive_exists(path).map_err(|()| {
        crate::services::private_store::error_codes::SYSTEM_PROMPT_WRITE.to_string()
    })? {
        return Ok(());
    }
    std::fs::rename(path, &archive).map_err(|_| {
        crate::services::private_store::error_codes::SYSTEM_PROMPT_WRITE.to_string()
    })?;
    crate::services::private_store::repair_path(&archive)
        .map_err(|_| crate::services::private_store::error_codes::SYSTEM_PROMPT_WRITE.to_string())
}
