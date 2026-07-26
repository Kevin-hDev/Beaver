use crate::models::GatewayConfig;
use crate::services::gateway::service::GatewayService;
use crate::services::gateway::token_probe::{self, ProbeEndpoints};
use crate::services::gateway::tokens;
use crate::services::gateway::tokens::AccountTokens;
use crate::services::gateway::types::{ChannelStatus, GatewayHealth};
use tauri::Emitter;

fn ensure_startable(config: &GatewayConfig) -> Result<(), String> {
    crate::services::gateway::config_validation::validate(config)?;
    if !config.enabled {
        return Err("Gateway désactivé".to_string());
    }
    Ok(())
}

async fn commit_after_validation<V, F>(validation: V, store: F) -> Result<(), String>
where
    V: std::future::Future<Output = Result<(), String>>,
    F: FnOnce() -> Result<(), String>,
{
    validation.await?;
    store()
}

#[tauri::command]
pub async fn gateway_status(
    state: tauri::State<'_, GatewayService>,
) -> Result<GatewayHealth, String> {
    Ok(state.health().await)
}

#[tauri::command]
pub async fn gateway_start(
    app: tauri::AppHandle,
    state: tauri::State<'_, GatewayService>,
) -> Result<(), String> {
    let config = state.config().await;
    ensure_startable(&config)?;
    state.start(config, app).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn disabled_gateway_cannot_start() {
        assert!(ensure_startable(&GatewayConfig::default()).is_err());
    }

    #[test]
    fn enabled_gateway_can_start() {
        let config = GatewayConfig {
            enabled: true,
            ..GatewayConfig::default()
        };
        assert!(ensure_startable(&config).is_ok());
    }

    #[tokio::test]
    async fn failed_token_probe_never_calls_storage() {
        let called = AtomicBool::new(false);
        let result = commit_after_validation(async { Err("échec".to_string()) }, || {
            called.store(true, Ordering::Relaxed);
            Ok(())
        })
        .await;

        assert!(result.is_err());
        assert!(!called.load(Ordering::Relaxed));
    }
}

#[tauri::command]
pub async fn gateway_stop(
    app: tauri::AppHandle,
    state: tauri::State<'_, GatewayService>,
) -> Result<(), String> {
    state.stop().await;
    let health = state.health().await;
    for channel in &health.channels {
        let _ = app.emit("gateway-channel-status", channel);
    }
    let _ = app.emit("gateway-status-changed", health);
    Ok(())
}

#[tauri::command]
pub async fn gateway_get_config(
    state: tauri::State<'_, GatewayService>,
) -> Result<GatewayConfig, String> {
    let disk = crate::services::config::read_config()
        .unwrap_or_default()
        .gateway;
    state.update_config(disk.clone()).await;
    Ok(disk)
}

#[tauri::command]
pub async fn gateway_set_config(
    app: tauri::AppHandle,
    state: tauri::State<'_, GatewayService>,
    config: GatewayConfig,
) -> Result<(), String> {
    crate::services::gateway::config_validation::validate(&config)?;
    let current_health = state.health().await;
    let should_restart = current_health.running
        && current_health
            .channels
            .iter()
            .any(|c| matches!(c.status, ChannelStatus::Starting | ChannelStatus::Running));
    let mut full = crate::services::config::read_config().unwrap_or_default();
    full.gateway = config.clone();
    crate::services::config::write_config(&full)?;
    state.update_config(config.clone()).await;

    if should_restart && config.enabled {
        return state.start(config, app).await;
    }
    if should_restart {
        state.stop().await;
    }
    let _ = app.emit("gateway-status-changed", state.health().await);
    Ok(())
}

#[tauri::command]
pub async fn gateway_configure_account_tokens(
    channel_id: String,
    account_id: String,
    credentials: AccountTokens,
) -> Result<(), String> {
    credentials
        .vault_entries(&channel_id, &account_id)
        .map_err(|_| "configuration des identifiants impossible".to_string())?;
    let endpoints = ProbeEndpoints::production();
    commit_after_validation(
        token_probe::validate_tokens(&channel_id, &credentials, &endpoints),
        || tokens::store_account_tokens(&channel_id, &account_id, &credentials),
    )
    .await
    .map_err(|_| "configuration des identifiants impossible".to_string())
}

#[tauri::command]
pub async fn gateway_delete_token(
    channel_id: String,
    account_id: String,
    token_kind: Option<String>,
) -> Result<(), String> {
    tokens::delete(&channel_id, &account_id, token_kind.as_deref())
}

#[tauri::command]
pub async fn gateway_has_token(
    channel_id: String,
    account_id: String,
    token_kind: String,
) -> Result<bool, String> {
    tokens::has(&channel_id, &account_id, &token_kind)
}
