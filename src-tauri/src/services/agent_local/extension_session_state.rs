use serde::{Deserialize, Serialize};
use std::io::Read;
use std::path::PathBuf;
use std::sync::LazyLock;
use tokio::sync::Mutex;

use super::extension_tool_selection::PluginDescriptor;

const STORE_MAX_BYTES: u64 = 64 * 1024;
const MAX_EPOCH_TEXT_CHARS: usize = 256;
static STORE_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct ExtensionSessionState {
    #[serde(default)]
    pub discovered_plugin_ids: Vec<String>,
    #[serde(default)]
    pub epoch: Option<DiscoveryEpoch>,
    #[serde(default)]
    pub plugin_tool_capacity: usize,
    #[serde(default)]
    pub plugin_descriptors: Vec<PluginDescriptor>,
    #[serde(default)]
    pub active_plugin_ids: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct DiscoveryEpoch {
    pub provider: String,
    pub model: String,
    pub context_window: u64,
    pub catalog_version: String,
    pub masked: bool,
}

pub async fn configure(
    session_id: &str,
    epoch: DiscoveryEpoch,
    computed_mask: bool,
    plugin_tool_capacity: usize,
    plugin_descriptors: Vec<PluginDescriptor>,
    preserve_dynamic_tools: bool,
) -> Result<ExtensionSessionState, String> {
    if invalid_epoch(&epoch) {
        return Err("État d'extensions invalide.".to_string());
    }
    mutate(session_id, |state| {
        if state
            .epoch
            .as_ref()
            .is_none_or(|current| !same_key(current, &epoch))
        {
            state.epoch = Some(DiscoveryEpoch {
                masked: computed_mask,
                ..epoch
            });
        }
        state.plugin_tool_capacity = plugin_tool_capacity;
        state.plugin_descriptors = plugin_descriptors;
        sanitize(state);
        super::extension_session_plugins::refresh_active(state, preserve_dynamic_tools);
        Ok(state.clone())
    })
    .await
}

pub async fn read(session_id: &str) -> Result<ExtensionSessionState, String> {
    super::session_store::validate_session_id(session_id)?;
    let _guard = STORE_LOCK.lock().await;
    Ok(load(session_id))
}

pub async fn mutate<T>(
    session_id: &str,
    update: impl FnOnce(&mut ExtensionSessionState) -> Result<T, String>,
) -> Result<T, String> {
    super::session_store::validate_session_id(session_id)?;
    let _guard = STORE_LOCK.lock().await;
    let mut state = load(session_id);
    let result = update(&mut state)?;
    sanitize(&mut state);
    save(session_id, &state).await?;
    Ok(result)
}

pub async fn remove(session_id: &str) -> Result<(), String> {
    super::session_store::validate_session_id(session_id)?;
    let _guard = STORE_LOCK.lock().await;
    match tokio::fs::remove_file(path(session_id)).await {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err("Suppression de l'état d'extensions impossible".into()),
    }
}

fn same_key(left: &DiscoveryEpoch, right: &DiscoveryEpoch) -> bool {
    left.provider == right.provider
        && left.model == right.model
        && left.context_window == right.context_window
        && left.catalog_version == right.catalog_version
}

fn sanitize(state: &mut ExtensionSessionState) {
    let mut seen = std::collections::HashSet::with_capacity(
        crate::services::extensions::MAX_DISCOVERED_PLUGINS,
    );
    state.discovered_plugin_ids.retain(|id| {
        if seen.len() >= crate::services::extensions::MAX_DISCOVERED_PLUGINS {
            return false;
        }
        crate::services::extensions::validate_identifier(id).is_ok() && seen.insert(id.clone())
    });
    if state.epoch.as_ref().is_some_and(invalid_epoch) {
        state.epoch = None;
    }
    state.plugin_tool_capacity = state
        .plugin_tool_capacity
        .min(crate::services::extensions::MAX_EXTENSION_TOOLS);
    super::extension_session_plugins::sanitize(state);
}

fn invalid_epoch(epoch: &DiscoveryEpoch) -> bool {
    invalid_epoch_text(&epoch.provider)
        || invalid_epoch_text(&epoch.model)
        || epoch.catalog_version.len() != 64
        || !epoch
            .catalog_version
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
}

fn invalid_epoch_text(value: &str) -> bool {
    value.is_empty() || value.chars().count() > MAX_EPOCH_TEXT_CHARS || value.contains('\0')
}

fn path(session_id: &str) -> PathBuf {
    crate::services::paths::data_dir()
        .join("extension-session-state")
        .join(format!("{session_id}.json"))
}

fn load(session_id: &str) -> ExtensionSessionState {
    let Ok(file) = std::fs::File::open(path(session_id)) else {
        return ExtensionSessionState::default();
    };
    let mut bytes = Vec::new();
    if file
        .take(STORE_MAX_BYTES.saturating_add(1))
        .read_to_end(&mut bytes)
        .is_err()
        || bytes.len() as u64 > STORE_MAX_BYTES
    {
        return ExtensionSessionState::default();
    }
    let mut state = serde_json::from_slice(&bytes).unwrap_or_default();
    sanitize(&mut state);
    state
}

async fn save(session_id: &str, state: &ExtensionSessionState) -> Result<(), String> {
    let bytes =
        serde_json::to_vec(state).map_err(|_| "État d'extensions indisponible.".to_string())?;
    if bytes.len() as u64 > STORE_MAX_BYTES {
        return Err("État d'extensions invalide.".to_string());
    }
    let path = path(session_id);
    let parent = path
        .parent()
        .ok_or_else(|| "État d'extensions indisponible.".to_string())?
        .to_path_buf();
    crate::services::private_store::ensure_private_dir_async(parent).await?;
    crate::services::private_store::atomic_write_async(path, bytes).await
}

#[cfg(test)]
#[path = "extension_session_state_tests.rs"]
mod tests;
