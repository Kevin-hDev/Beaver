use std::fs::File;
use std::io::Read;
use std::path::Path;

use super::config::{self, StoredConnector, ENDPOINT_NOT_ALLOWED, MAX_CONNECTORS};
use super::{config_migration, trusted};

const MAX_CONFIG_FILE_BYTES: u64 = 256 * 1024;

pub(super) fn parse_file(path: &Path) -> Result<Vec<StoredConnector>, String> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(_) => return Err("lecture connecteurs impossible".to_string()),
    };
    let mut content = String::new();
    file.take(MAX_CONFIG_FILE_BYTES + 1)
        .read_to_string(&mut content)
        .map_err(|_| "configuration MCP invalide".to_string())?;
    if content.len() as u64 > MAX_CONFIG_FILE_BYTES {
        return Err("configuration MCP invalide".to_string());
    }
    let connectors: Vec<StoredConnector> =
        serde_json::from_str(&content).map_err(|_| "configuration MCP invalide".to_string())?;
    if connectors.len() > MAX_CONNECTORS {
        return Err("limite de connecteurs atteinte".to_string());
    }
    validate_unique_ids(&connectors)?;
    Ok(connectors)
}

pub(super) fn load(
    path: &Path,
    allow_invalid_known_endpoint: bool,
) -> Result<(Vec<StoredConnector>, bool), String> {
    let mut connectors = parse_file(path)?;
    let migrated = config_migration::normalize_list(&mut connectors);
    for connector in &connectors {
        if let Err(error) = config::validate_connector(connector) {
            if !allow_invalid_known_endpoint
                || error != ENDPOINT_NOT_ALLOWED
                || trusted::canonical_endpoint(&connector.id).is_none()
            {
                return Err(error);
            }
        }
    }
    Ok((connectors, migrated))
}

pub(super) fn validate_unique_ids(connectors: &[StoredConnector]) -> Result<(), String> {
    for (index, connector) in connectors.iter().enumerate() {
        if connectors[..index]
            .iter()
            .any(|prior| prior.id == connector.id)
        {
            return Err("configuration MCP invalide".to_string());
        }
    }
    Ok(())
}
