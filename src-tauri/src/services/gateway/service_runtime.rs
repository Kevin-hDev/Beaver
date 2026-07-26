use std::sync::Arc;
use std::time::Duration;

use tauri::Emitter;
use tokio::sync::{mpsc, RwLock};

use super::channels::{ChannelAdapter, ChannelContext, InboundMessage};
use super::security::ids;
use super::service_audit;
use super::service_state::{build_health, GatewayState};
use super::supervisor::{ChannelSupervisor, RestartDecision};
use super::types::{ChannelHealthEntry, ChannelKey, ChannelStatus};
use super::{tokens, watchdog::StallWatchdog};
use crate::models::ChannelAccountConfig;

pub(crate) fn validate_account(channel_id: &str, acc: &ChannelAccountConfig) -> Result<(), String> {
    ids::validate_channel_id(channel_id)?;
    ids::validate_account_id(&acc.account_id)?;
    if acc.provider.trim().is_empty() || acc.model.trim().is_empty() {
        return Err("provider ou modèle manquant".to_string());
    }
    for user in &acc.allowlist {
        ids::validate_external_id(user)?;
    }
    for kind in tokens::required_kinds(channel_id) {
        if !tokens::has(channel_id, &acc.account_id, kind)? {
            return Err("token manquant".to_string());
        }
    }
    Ok(())
}

pub(crate) async fn run_supervised_channel(
    adapter: Arc<dyn ChannelAdapter>,
    ctx: ChannelContext,
    sender: mpsc::Sender<InboundMessage>,
    state: Arc<RwLock<GatewayState>>,
    key: ChannelKey,
    app: tauri::AppHandle,
) {
    let mut supervisor = ChannelSupervisor::new(&key.channel_id, &key.account_id);
    let watchdog = StallWatchdog::spawn(Duration::from_secs(180), |_| {});
    watchdog.arm();
    loop {
        if ctx.cancel.is_cancelled() {
            watchdog.stop();
            return;
        }
        set_status(&state, &app, &key, ChannelStatus::Starting, None).await;
        let start_result = match adapter.validate_config(&ctx.config).await {
            Ok(()) => adapter.start(ctx.clone(), sender.clone()).await,
            Err(e) => Err(e),
        };
        match start_result {
            Ok(handle) => {
                if !handle_channel_run(handle, &mut supervisor, &state, &app, &key, &ctx).await {
                    watchdog.stop();
                    return;
                }
            }
            Err(e) => {
                if service_audit::auth_error(&key, &e).is_err() {
                    set_status(
                        &state,
                        &app,
                        &key,
                        ChannelStatus::Error,
                        Some("auditUnavailable"),
                    )
                    .await;
                    watchdog.stop();
                    return;
                }
                if !handle_restart(&mut supervisor, &state, &app, &key, e.is_auth).await {
                    watchdog.stop();
                    return;
                }
            }
        }
    }
}

async fn handle_channel_run(
    handle: tokio::task::JoinHandle<()>,
    supervisor: &mut ChannelSupervisor,
    state: &Arc<RwLock<GatewayState>>,
    app: &tauri::AppHandle,
    key: &ChannelKey,
    ctx: &ChannelContext,
) -> bool {
    supervisor.mark_started();
    if service_audit::channel_started(key).is_err() {
        handle.abort();
        set_status(
            state,
            app,
            key,
            ChannelStatus::Error,
            Some("auditUnavailable"),
        )
        .await;
        return false;
    }
    set_status(state, app, key, ChannelStatus::Running, None).await;
    let _ = handle.await;
    if ctx.cancel.is_cancelled() {
        return false;
    }
    handle_restart(supervisor, state, app, key, false).await
}

async fn handle_restart(
    supervisor: &mut ChannelSupervisor,
    state: &Arc<RwLock<GatewayState>>,
    app: &tauri::AppHandle,
    key: &ChannelKey,
    is_auth: bool,
) -> bool {
    match supervisor.on_error(is_auth) {
        RestartDecision::Retry(delay) => {
            set_status(state, app, key, ChannelStatus::Starting, None).await;
            tokio::time::sleep(delay).await;
            true
        }
        RestartDecision::GiveUp(reason) => {
            let audit_failed =
                service_audit::channel_stopped(key, Some("restart_give_up"), Some(&reason))
                    .is_err();
            let code = if audit_failed {
                "auditUnavailable"
            } else {
                "unavailable"
            };
            set_status(state, app, key, ChannelStatus::Error, Some(code)).await;
            false
        }
    }
}

async fn set_status(
    state: &Arc<RwLock<GatewayState>>,
    app: &tauri::AppHandle,
    key: &ChannelKey,
    status: ChannelStatus,
    error: Option<&str>,
) {
    let mut s = state.write().await;
    if let Some(entry) = s.channels.get_mut(key) {
        entry.status = status;
        entry.error = error.map(str::to_string);
    }
    let _ = app.emit("gateway-status-changed", build_health(&s));
    emit_channel_status(app, key, status, error);
}

pub(crate) fn emit_channel_status(
    app: &tauri::AppHandle,
    key: &ChannelKey,
    status: ChannelStatus,
    error: Option<&str>,
) {
    let _ = app.emit(
        "gateway-channel-status",
        ChannelHealthEntry {
            channel_id: key.channel_id.clone(),
            account_id: key.account_id.clone(),
            status,
            error: error.map(str::to_string),
        },
    );
}
