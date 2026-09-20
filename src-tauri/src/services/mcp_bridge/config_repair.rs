use std::fs::File;
use std::io::Read;
use std::path::Path;

use super::config::{self, StoredConnector, ENDPOINT_NOT_ALLOWED, MAX_CONNECTORS};
use super::config_migration;

const MAX_REPAIR_FILE_BYTES: u64 = 256 * 1024;

pub(super) fn load_from_path(path: &Path) -> Result<Vec<StoredConnector>, String> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(_) => return Err("lecture connecteurs impossible".to_string()),
    };
    let mut content = String::new();
    file.take(MAX_REPAIR_FILE_BYTES + 1)
        .read_to_string(&mut content)
        .map_err(|_| "configuration MCP invalide".to_string())?;
    if content.len() as u64 > MAX_REPAIR_FILE_BYTES {
        return Err("configuration MCP invalide".to_string());
    }
    let mut connectors: Vec<StoredConnector> =
        serde_json::from_str(&content).map_err(|_| "configuration MCP invalide".to_string())?;
    if connectors.len() > MAX_CONNECTORS {
        return Err("limite de connecteurs atteinte".to_string());
    }
    config_migration::normalize_list(&mut connectors);
    for (index, connector) in connectors.iter().enumerate() {
        if connectors[..index]
            .iter()
            .any(|prior| prior.id == connector.id)
        {
            return Err("configuration MCP invalide".to_string());
        }
        if let Err(error) = config::validate_connector(connector) {
            if error != ENDPOINT_NOT_ALLOWED {
                return Err(error);
            }
        }
    }
    Ok(connectors)
}

fn removal_from_path(
    path: &Path,
    connector_id: &str,
) -> Result<(Option<StoredConnector>, Vec<StoredConnector>), String> {
    config::validate_connector_id(connector_id)?;
    let mut connectors = load_from_path(path)?;
    let position = connectors.iter().position(|item| item.id == connector_id);
    let removed = position.map(|index| connectors.remove(index));
    for connector in &connectors {
        config::validate_connector(connector)?;
    }
    Ok((removed, connectors))
}

pub(super) fn preview_remove_from_path(
    path: &Path,
    connector_id: &str,
) -> Result<Option<StoredConnector>, String> {
    removal_from_path(path, connector_id).map(|(removed, _)| removed)
}

pub(super) fn remove_from_path(path: &Path, connector_id: &str) -> Result<bool, String> {
    let (removed, remaining) = removal_from_path(path, connector_id)?;
    if removed.is_some() {
        config::save_to_path(path, &remaining)?;
        return Ok(true);
    }
    Ok(false)
}
