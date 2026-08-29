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
            validate_key_for_connection(ProviderConnectionKind::QwenModelStudio, key)?;
            let connection =
                connection.ok_or_else(|| "provider_configuration_invalid".to_string())?;
            let encoded = qwen::encode(connection)?;
            api_keys::set_key_with_raw(provider, key, &[(qwen::VAULT_KEY, encoded.as_str())])
        }
        ProviderConnectionKind::ApiKey if connection.is_none() => api_keys::set_key(provider, key),
        ProviderConnectionKind::ApiKey => Err("provider_configuration_invalid".to_string()),
    }
}

fn validate_key_for_connection(kind: ProviderConnectionKind, key: &str) -> Result<(), String> {
    match kind {
        ProviderConnectionKind::QwenModelStudio => api_keys::reject_unsupported_qwen_key(key),
        ProviderConnectionKind::ApiKey => Ok(()),
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
pub async fn get_provider_connection(
    provider: String,
) -> Result<Option<QwenConnectionInput>, String> {
    match connection_kind(&provider)? {
        ProviderConnectionKind::QwenModelStudio => {
            if !api_keys::has_key(&provider) {
                return Ok(None);
            }
            qwen::load().map(|record| Some(record.connection))
        }
        ProviderConnectionKind::ApiKey => Ok(None),
    }
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

#[cfg(test)]
mod tests {
    #[test]
    fn unsupported_qwen_subscription_key_is_rejected_at_the_save_boundary() {
        assert!(super::validate_key_for_connection(
            crate::models::provider_contract::ProviderConnectionKind::QwenModelStudio,
            "sk-sp-fixture",
        )
        .is_err());
    }
}
