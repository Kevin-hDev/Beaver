use std::path::Path;

use super::config::{self, StoredConnector};
use super::config_read;

fn removal_from_path(
    path: &Path,
    connector_id: &str,
) -> Result<(Option<StoredConnector>, Vec<StoredConnector>), String> {
    config::validate_connector_id(connector_id)?;
    let (mut connectors, _) = config_read::load(path, true)?;
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
