use super::config::StoredConnector;
use super::registry::{self, IdentityMutation};

pub(crate) fn commit_after_probe(
    connector_id: &str,
    expected_generation: Option<u64>,
    probe: Result<(), String>,
    load_previous: impl FnOnce() -> Result<Option<StoredConnector>, String>,
    store_config: impl FnOnce() -> Result<(), String>,
    store_secrets: impl FnOnce(&IdentityMutation) -> Result<(), String>,
    rollback_config: impl FnOnce(Option<StoredConnector>) -> Result<(), String>,
) -> Result<(), String> {
    probe?;
    let commit = |mutation: &IdentityMutation| {
        let previous = load_previous()?;
        store_config()?;
        if let Err(error) = store_secrets(mutation) {
            rollback_config(previous).map_err(|_| "configuration MCP indisponible")?;
            return Err(error);
        }
        Ok(())
    };
    match expected_generation {
        Some(generation) => {
            registry::mutate_identity_if_generation(connector_id, generation, commit)
        }
        None => registry::mutate_identity(connector_id, commit),
    }
}
