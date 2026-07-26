use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub use super::env_keys::validated_env_keys;
use super::{config_migration, stdio_catalog, stdio_cmd, trusted};

pub const MAX_CONNECTORS: usize = 32;
const FILENAME: &str = "mcp-connectors.json";

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct StoredConnector {
    pub id: String,
    pub status: String,
    pub enabled_in_chat: bool,
    pub endpoint: Option<String>,
    pub install_command: Option<String>,
    #[serde(default)]
    pub env_keys: Option<Vec<String>>,
}

pub fn load() -> Result<Vec<StoredConnector>, String> {
    load_from_path(&storage_path())
}

pub fn find(connector_id: &str) -> Result<Option<StoredConnector>, String> {
    validate_connector_id(connector_id)?;
    Ok(load()?
        .into_iter()
        .find(|connector| connector.id == connector_id))
}

pub fn upsert(connector: StoredConnector) -> Result<(), String> {
    validate_connector(&connector)?;
    let path = storage_path();
    let mut list = load_from_path(&path)?;
    if let Some(existing) = list.iter_mut().find(|c| c.id == connector.id) {
        *existing = connector;
    } else {
        if list.len() >= MAX_CONNECTORS {
            return Err("limite de connecteurs atteinte".to_string());
        }
        list.push(connector);
    }
    save_to_path(&path, &list)
}

pub fn remove(connector_id: &str) -> Result<bool, String> {
    validate_connector_id(connector_id)?;
    let path = storage_path();
    let before = load_from_path(&path)?;
    let after: Vec<StoredConnector> = before
        .iter()
        .filter(|c| c.id != connector_id)
        .cloned()
        .collect();
    let removed = before.len() != after.len();
    if removed {
        save_to_path(&path, &after)?;
    }
    Ok(removed)
}

pub fn set_status(connector_id: &str, status: &str) -> Result<(), String> {
    if !is_valid_status(status) {
        return Err("statut invalide".to_string());
    }
    update(connector_id, |c| c.status = status.to_string())
}

pub fn set_chat_enabled(connector_id: &str, enabled: bool) -> Result<(), String> {
    update(connector_id, |c| c.enabled_in_chat = enabled)
}

pub fn validate_connector(c: &StoredConnector) -> Result<(), String> {
    validate_connector_id(&c.id)?;
    if !is_valid_status(&c.status) {
        return Err("statut invalide".to_string());
    }
    if let Some(endpoint) = &c.endpoint {
        if !trusted::is_trusted_endpoint_for_connector(&c.id, endpoint) {
            return Err("endpoint MCP non autorisé".to_string());
        }
    }
    if let Some(cmd) = install_command_for(c) {
        stdio_cmd::parse_install_command(&c.id, &cmd)?;
    }
    let _ = validated_env_keys(c.env_keys.as_deref())?;
    if c.endpoint.is_none() && c.install_command.is_none() {
        return Err("connecteur MCP incomplet".to_string());
    }
    Ok(())
}

pub fn validate_connector_id(id: &str) -> Result<(), String> {
    if is_valid_connector_id(id) {
        Ok(())
    } else {
        Err("identifiant invalide".to_string())
    }
}

pub fn is_valid_connector_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

pub fn install_command_for(c: &StoredConnector) -> Option<String> {
    stdio_catalog::install_command(&c.id).or_else(|| c.install_command.clone())
}

fn update(connector_id: &str, apply: impl FnOnce(&mut StoredConnector)) -> Result<(), String> {
    validate_connector_id(connector_id)?;
    let path = storage_path();
    let mut list = load_from_path(&path)?;
    let connector = list
        .iter_mut()
        .find(|c| c.id == connector_id)
        .ok_or("connecteur introuvable")?;
    apply(connector);
    validate_connector(connector)?;
    save_to_path(&path, &list)
}

pub(crate) fn load_from_path(path: &Path) -> Result<Vec<StoredConnector>, String> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(_) => return Err("lecture connecteurs impossible".to_string()),
    };
    let mut parsed: Vec<StoredConnector> =
        serde_json::from_str(&content).map_err(|_| "configuration MCP invalide".to_string())?;
    if parsed.len() > MAX_CONNECTORS {
        return Err("limite de connecteurs atteinte".to_string());
    }
    let migrated = config_migration::normalize_list(&mut parsed);
    for connector in &parsed {
        validate_connector(connector)?;
    }
    if migrated {
        save_to_path(path, &parsed)?;
    }
    Ok(parsed)
}

pub(crate) fn save_to_path(path: &Path, list: &[StoredConnector]) -> Result<(), String> {
    if list.len() > MAX_CONNECTORS {
        return Err("limite de connecteurs atteinte".to_string());
    }
    for connector in list {
        validate_connector(connector)?;
    }
    let data = serde_json::to_vec_pretty(list).map_err(|_| "sérialisation échouée")?;
    crate::services::private_store::atomic_write(path, &data)
        .map_err(|_| "écriture connecteurs impossible".to_string())
}

fn storage_path() -> PathBuf {
    crate::services::paths::data_dir().join(FILENAME)
}

fn is_valid_status(status: &str) -> bool {
    status == "connected" || status == "disconnected"
}
