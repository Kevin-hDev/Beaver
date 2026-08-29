//! Commandes Tauri pour la gestion des clés API.
//!
//! IMPORTANT : aucune commande ne retourne la clé en clair au frontend.
//! set/delete/has/list/test seulement.

use crate::services::api_keys;
use crate::{
    models::provider_contract::{ProviderConnectionKind, QwenConnectionInput},
    services::provider_connections::qwen,
};
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
    match connection_kind(provider)? {
        ProviderConnectionKind::QwenModelStudio => {
            let connection =
                connection.ok_or_else(|| "provider_configuration_invalid".to_string())?;
            let encoded = qwen::encode(connection)?;
            api_keys::set_key_with_raw(provider, key, &[(qwen::VAULT_KEY, encoded.as_str())])
        }
        ProviderConnectionKind::ApiKey if connection.is_none() => api_keys::set_key(provider, key),
        ProviderConnectionKind::ApiKey => Err("provider_configuration_invalid".to_string()),
    }
}

fn connection_kind(provider: &str) -> Result<ProviderConnectionKind, String> {
    crate::services::llm::catalog::find_configurable(provider)
        .map(|spec| spec.connection_kind)
        .ok_or_else(|| "provider_configuration_invalid".to_string())
}

#[tauri::command]
pub async fn delete_api_key(app: tauri::AppHandle, provider: String) -> Result<(), String> {
    match connection_kind(&provider)? {
        ProviderConnectionKind::QwenModelStudio => {
            api_keys::delete_key_with_raw(&provider, &[qwen::VAULT_KEY])?
        }
        ProviderConnectionKind::ApiKey => api_keys::delete_key(&provider)?,
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
    match connection_kind(&provider)? {
        ProviderConnectionKind::QwenModelStudio => {
            let connection =
                connection.ok_or_else(|| "provider_configuration_invalid".to_string())?;
            api_keys::test_qwen_key_raw(&key, &connection).await
        }
        ProviderConnectionKind::ApiKey if connection.is_none() => {
            api_keys::test_key_raw(&provider, &key).await
        }
        ProviderConnectionKind::ApiKey => Err("provider_configuration_invalid".to_string()),
    }
}
