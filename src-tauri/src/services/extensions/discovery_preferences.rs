use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::io::Read;
use std::path::PathBuf;
use std::sync::{LazyLock, Mutex};

use super::discovery_limits::{DISCOVERY_STORE_MAX_BYTES, MAX_USER_PRIORITY_PLUGINS};
use super::types::ExtensionRecord;

static STORE_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DiscoveryPreferences {
    #[serde(default)]
    pub protected_plugin_ids: Vec<String>,
}

pub fn get() -> Result<DiscoveryPreferences, String> {
    let _guard = STORE_LOCK
        .lock()
        .map_err(|_| "Préférences d'extensions indisponibles.".to_string())?;
    Ok(load())
}

pub fn set(plugin_ids: Vec<String>) -> Result<DiscoveryPreferences, String> {
    validate_requested(&plugin_ids)?;
    let enabled = enabled_ids(&super::registry::list()?);
    if plugin_ids.iter().any(|id| !enabled.contains(id)) {
        return Err("Sélection de plugins invalide.".to_string());
    }
    let preferences = DiscoveryPreferences {
        protected_plugin_ids: plugin_ids,
    };
    {
        let _guard = STORE_LOCK
            .lock()
            .map_err(|_| "Préférences d'extensions indisponibles.".to_string())?;
        save(&preferences)?;
    }
    super::registry::refresh_index()?;
    Ok(preferences)
}

pub fn sanitize(records: &[ExtensionRecord]) -> Result<DiscoveryPreferences, String> {
    let enabled = enabled_ids(records);
    let _guard = STORE_LOCK
        .lock()
        .map_err(|_| "Préférences d'extensions indisponibles.".to_string())?;
    let current = load();
    let mut seen = HashSet::with_capacity(MAX_USER_PRIORITY_PLUGINS);
    let protected_plugin_ids = current
        .protected_plugin_ids
        .iter()
        .filter(|id| enabled.contains(*id) && seen.insert((*id).clone()))
        .take(MAX_USER_PRIORITY_PLUGINS)
        .cloned()
        .collect::<Vec<_>>();
    let next = DiscoveryPreferences {
        protected_plugin_ids,
    };
    if next != current {
        save(&next)?;
    }
    Ok(next)
}

fn validate_requested(plugin_ids: &[String]) -> Result<(), String> {
    if plugin_ids.len() > MAX_USER_PRIORITY_PLUGINS {
        return Err("Trop de plugins prioritaires.".to_string());
    }
    let mut seen = HashSet::with_capacity(plugin_ids.len());
    for id in plugin_ids {
        super::validation::identifier(id)?;
        if !seen.insert(id) {
            return Err("Sélection de plugins invalide.".to_string());
        }
    }
    Ok(())
}

fn enabled_ids(records: &[ExtensionRecord]) -> HashSet<String> {
    records
        .iter()
        .filter(|record| record.enabled)
        .map(|record| record.manifest.id.clone())
        .collect()
}

fn path() -> PathBuf {
    crate::services::paths::data_dir().join("extension-discovery-preferences.json")
}

fn load() -> DiscoveryPreferences {
    let Some(bytes) = read_bounded() else {
        return DiscoveryPreferences::default();
    };
    serde_json::from_slice(&bytes).unwrap_or_default()
}

fn read_bounded() -> Option<Vec<u8>> {
    let file = std::fs::File::open(path()).ok()?;
    let mut bytes = Vec::new();
    file.take(DISCOVERY_STORE_MAX_BYTES.saturating_add(1))
        .read_to_end(&mut bytes)
        .ok()?;
    (bytes.len() as u64 <= DISCOVERY_STORE_MAX_BYTES).then_some(bytes)
}

fn save(preferences: &DiscoveryPreferences) -> Result<(), String> {
    let bytes = serde_json::to_vec(preferences)
        .map_err(|_| "Préférences d'extensions indisponibles.".to_string())?;
    if bytes.len() as u64 > DISCOVERY_STORE_MAX_BYTES {
        return Err("Préférences d'extensions invalides.".to_string());
    }
    crate::services::private_store::atomic_write(&path(), &bytes)
        .map_err(|_| "Préférences d'extensions indisponibles.".to_string())
}

#[cfg(test)]
#[path = "discovery_preferences_tests.rs"]
mod tests;
