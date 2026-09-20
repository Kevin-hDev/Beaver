use crate::services::api_keys;
use crate::services::mcp_bridge::{config, env_tokens, process_manager, registry};
use tauri::Emitter;

#[tauri::command]
pub async fn list_mcp_connectors() -> Result<Vec<config::StoredConnector>, String> {
    config::load()
}

#[tauri::command]
pub async fn add_mcp_connector(
    app: tauri::AppHandle,
    connector: config::StoredConnector,
) -> Result<(), String> {
    let connector_id = connector.id.clone();
    config::validate_connector(&connector)?;
    registry::mutate_identity(&connector_id, |_| config::upsert(connector))?;
    let _ = app.emit("fs:connectors-changed", ());
    Ok(())
}

#[tauri::command]
pub async fn remove_mcp_connector(
    app: tauri::AppHandle,
    connector_id: String,
) -> Result<(), String> {
    registry::mutate_identity(&connector_id, |_| {
        let connector = config::find(&connector_id)?;
        delete_connector_secrets(&connector_id, connector.as_ref())?;
        config::remove(&connector_id).map(|_| ())
    })?;
    process_manager::shutdown_one(&connector_id).await;
    let _ = app.emit("fs:connectors-changed", ());
    Ok(())
}

#[tauri::command]
pub async fn set_mcp_connector_status(
    app: tauri::AppHandle,
    connector_id: String,
    status: String,
) -> Result<(), String> {
    config::validate_status(&status)?;
    config::find(&connector_id)?.ok_or("connecteur introuvable")?;
    registry::mutate_identity(&connector_id, |_| {
        config::set_status(&connector_id, &status)
    })?;
    if status == "disconnected" {
        process_manager::shutdown_one(&connector_id).await;
    }
    let _ = app.emit("fs:connectors-changed", ());
    Ok(())
}

#[tauri::command]
pub async fn set_mcp_connector_chat_enabled(
    app: tauri::AppHandle,
    connector_id: String,
    enabled: bool,
) -> Result<(), String> {
    config::find(&connector_id)?.ok_or("connecteur introuvable")?;
    registry::mutate_identity(&connector_id, |_| {
        config::set_chat_enabled(&connector_id, enabled)
    })?;
    let _ = app.emit("fs:connectors-changed", ());
    Ok(())
}

#[tauri::command]
pub async fn test_mcp_connector(connector: config::StoredConnector) -> Result<(), String> {
    registry::test_connector(connector).await
}

#[tauri::command]
pub async fn configure_mcp_connector_tokens(
    app: tauri::AppHandle,
    connector: config::StoredConnector,
    env_tokens: Vec<env_tokens::EnvTokenInput>,
) -> Result<(), String> {
    env_tokens::validate(&connector, &env_tokens)?;
    let transient = env_tokens::owned_pairs(&env_tokens);
    let probe = registry::test_connector_with_env(connector.clone(), transient).await;

    let vault_keys: Vec<String> = env_tokens
        .iter()
        .map(|token| env_tokens::vault_key(&connector.id, &token.env_key))
        .collect();
    let entries: Vec<(&str, &str)> = vault_keys
        .iter()
        .zip(&env_tokens)
        .map(|(key, token)| (key.as_str(), token.value.as_str()))
        .collect();
    commit_after_probe(
        &connector.id,
        probe,
        || config::find(&connector.id),
        || config::upsert(connector.clone()),
        || api_keys::set_raw_batch(&entries),
        |previous| config::restore(&connector.id, previous),
    )?;
    let _ = app.emit("fs:connectors-changed", ());
    Ok(())
}

fn commit_after_probe(
    connector_id: &str,
    probe: Result<(), String>,
    load_previous: impl FnOnce() -> Result<Option<config::StoredConnector>, String>,
    store_config: impl FnOnce() -> Result<(), String>,
    store_secrets: impl FnOnce() -> Result<(), String>,
    rollback_config: impl FnOnce(Option<config::StoredConnector>) -> Result<(), String>,
) -> Result<(), String> {
    probe?;
    registry::mutate_identity(connector_id, |_| {
        let previous = load_previous()?;
        store_config()?;
        if let Err(error) = store_secrets() {
            rollback_config(previous).map_err(|_| "configuration MCP indisponible")?;
            return Err(error);
        }
        Ok(())
    })
}

fn delete_connector_secrets(
    connector_id: &str,
    connector: Option<&config::StoredConnector>,
) -> Result<(), String> {
    let env_keys = connector
        .map(|value| config::validated_env_keys(value.env_keys.as_deref()))
        .transpose()?
        .unwrap_or_default();
    let vault_keys: Vec<String> = env_keys
        .iter()
        .map(|env_key| env_tokens::vault_key(connector_id, env_key))
        .collect();
    let refs: Vec<&str> = vault_keys.iter().map(String::as_str).collect();
    api_keys::delete_mcp_bundle(connector_id, &refs)
}

#[cfg(test)]
mod tests {
    use super::commit_after_probe;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn failed_probe_never_changes_secrets_or_configuration() {
        let secret_write = AtomicBool::new(false);
        let config_write = AtomicBool::new(false);
        let result = commit_after_probe(
            "test-connector",
            Err("probe failed".to_string()),
            || Ok(None),
            || {
                secret_write.store(true, Ordering::SeqCst);
                Ok(())
            },
            || {
                config_write.store(true, Ordering::SeqCst);
                Ok(())
            },
            |_| Ok(()),
        );
        assert!(result.is_err());
        assert!(!secret_write.load(Ordering::SeqCst));
        assert!(!config_write.load(Ordering::SeqCst));
    }

    #[test]
    fn failed_secret_write_rolls_back_configuration() {
        let rollback = AtomicBool::new(false);
        let result = commit_after_probe(
            "test-connector",
            Ok(()),
            || Ok(None),
            || Ok(()),
            || Err("vault failed".to_string()),
            |_| {
                rollback.store(true, Ordering::SeqCst);
                Ok(())
            },
        );
        assert!(result.is_err());
        assert!(rollback.load(Ordering::SeqCst));
    }

    #[test]
    fn failed_rollback_is_reported_as_configuration_unavailable() {
        let result = commit_after_probe(
            "test-connector",
            Ok(()),
            || Ok(None),
            || Ok(()),
            || Err("vault failed".to_string()),
            |_| Err("restore failed".to_string()),
        );
        assert_eq!(result.unwrap_err(), "configuration MCP indisponible");
    }
}
