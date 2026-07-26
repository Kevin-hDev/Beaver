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
    config::upsert(connector)?;
    registry::invalidate_cache(&connector_id);
    let _ = app.emit("fs:connectors-changed", ());
    Ok(())
}

#[tauri::command]
pub async fn remove_mcp_connector(
    app: tauri::AppHandle,
    connector_id: String,
) -> Result<(), String> {
    let connector = config::find(&connector_id)?;
    delete_connector_secrets(&connector_id, connector.as_ref())?;
    config::remove(&connector_id)?;
    registry::invalidate_cache(&connector_id);
    process_manager::shutdown_one(&connector_id);
    let _ = app.emit("fs:connectors-changed", ());
    Ok(())
}

#[tauri::command]
pub async fn set_mcp_connector_status(
    app: tauri::AppHandle,
    connector_id: String,
    status: String,
) -> Result<(), String> {
    config::set_status(&connector_id, &status)?;
    registry::invalidate_cache(&connector_id);
    if status == "disconnected" {
        process_manager::shutdown_one(&connector_id);
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
    config::set_chat_enabled(&connector_id, enabled)?;
    registry::invalidate_cache(&connector_id);
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
    let previous = config::find(&connector.id)?;
    commit_after_probe(
        probe,
        || config::upsert(connector.clone()),
        || api_keys::set_raw_batch(&entries),
        || restore_connector(&connector.id, previous),
    )?;
    registry::invalidate_cache(&connector.id);
    let _ = app.emit("fs:connectors-changed", ());
    Ok(())
}

fn commit_after_probe(
    probe: Result<(), String>,
    store_config: impl FnOnce() -> Result<(), String>,
    store_secrets: impl FnOnce() -> Result<(), String>,
    rollback_config: impl FnOnce() -> Result<(), String>,
) -> Result<(), String> {
    probe?;
    store_config()?;
    if let Err(error) = store_secrets() {
        let _ = rollback_config();
        return Err(error);
    }
    Ok(())
}

fn restore_connector(
    connector_id: &str,
    previous: Option<config::StoredConnector>,
) -> Result<(), String> {
    match previous {
        Some(connector) => config::upsert(connector),
        None => config::remove(connector_id).map(|_| ()),
    }
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
            Err("probe failed".to_string()),
            || {
                secret_write.store(true, Ordering::SeqCst);
                Ok(())
            },
            || {
                config_write.store(true, Ordering::SeqCst);
                Ok(())
            },
            || Ok(()),
        );
        assert!(result.is_err());
        assert!(!secret_write.load(Ordering::SeqCst));
        assert!(!config_write.load(Ordering::SeqCst));
    }

    #[test]
    fn failed_secret_write_rolls_back_configuration() {
        let rollback = AtomicBool::new(false);
        let result = commit_after_probe(
            Ok(()),
            || Ok(()),
            || Err("vault failed".to_string()),
            || {
                rollback.store(true, Ordering::SeqCst);
                Ok(())
            },
        );
        assert!(result.is_err());
        assert!(rollback.load(Ordering::SeqCst));
    }
}
