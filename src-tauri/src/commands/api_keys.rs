//! Commandes Tauri pour la gestion des clés API.
//!
//! IMPORTANT : aucune commande ne retourne la clé en clair au frontend.
//! set/delete/has/list/test seulement.

use crate::services::api_keys;
use crate::{models::provider_contract::QwenConnectionInput, services::provider_connections::qwen};
use tauri::Emitter;
use zeroize::{Zeroize, Zeroizing};

#[tauri::command]
pub async fn set_api_key(
    app: tauri::AppHandle,
    provider: String,
    mut key: String,
    connection: Option<QwenConnectionInput>,
) -> Result<(), String> {
    let result = set_provider_key(&provider, &key, connection);
    key.zeroize();
    if result.is_ok() {
        crate::services::provider_usage::invalidate_remote(&provider).await;
        let _ = app.emit("providers-changed", ());
    }
    result
}

fn set_provider_key(
    provider: &str,
    key: &str,
    connection: Option<QwenConnectionInput>,
) -> Result<(), String> {
    if provider == "qwen" {
        let connection = connection.ok_or_else(|| "provider_configuration_invalid".to_string())?;
        let encoded = qwen::encode(connection)?;
        return api_keys::set_key_with_raw(provider, key, &[(qwen::VAULT_KEY, encoded.as_str())]);
    }
    if connection.is_some() {
        return Err("provider_configuration_invalid".to_string());
    }
    api_keys::set_key(provider, key)
}

#[tauri::command]
pub async fn delete_api_key(app: tauri::AppHandle, provider: String) -> Result<(), String> {
    if provider == "qwen" {
        api_keys::delete_key_with_raw(&provider, &[qwen::VAULT_KEY])?;
    } else {
        api_keys::delete_key(&provider)?;
    }
    crate::services::provider_usage::invalidate_remote(&provider).await;
    let _ = app.emit("providers-changed", ());
    Ok(())
}

#[tauri::command]
pub async fn has_api_key(provider: String) -> Result<bool, String> {
    Ok(api_keys::has_key(&provider))
}

#[tauri::command]
pub async fn list_configured_providers() -> Result<Vec<String>, String> {
    Ok(api_keys::list_configured())
}

#[tauri::command]
pub async fn test_api_key(provider: String) -> Result<(), String> {
    api_keys::test_key(&provider).await
}

#[tauri::command]
pub async fn test_api_key_with_value(
    provider: String,
    key: String,
    connection: Option<QwenConnectionInput>,
) -> Result<(), String> {
    let key = Zeroizing::new(key);
    if provider == "qwen" {
        let connection = connection.ok_or_else(|| "provider_configuration_invalid".to_string())?;
        api_keys::test_qwen_key_raw(&key, &connection).await
    } else if connection.is_some() {
        Err("provider_configuration_invalid".to_string())
    } else {
        api_keys::test_key_raw(&provider, &key).await
    }
}
