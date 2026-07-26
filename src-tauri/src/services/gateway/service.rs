use std::sync::Arc;
use tauri::Emitter;
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio_util::sync::CancellationToken;

use super::agent_bridge::GatewayAgentBridge;
use super::channels::telegram::TelegramAdapter;
use super::channels::{ChannelAdapter, ChannelContext, InboundMessage};
use super::security::audit;
use super::security::rate_state::GatewayRateLimiters;
use super::service_audit;
use super::service_runtime::{emit_channel_status, run_supervised_channel, validate_account};
use super::service_state::{build_health, shared_state, ChannelEntry, GatewayState};
use super::types::{ChannelKey, ChannelStatus, GatewayHealth};
use crate::models::{ChannelAccountConfig, GatewayConfig};

pub struct GatewayService {
    pub(crate) state: Arc<RwLock<GatewayState>>,
}

impl GatewayService {
    pub fn new() -> Self {
        Self {
            state: shared_state(),
        }
    }

    pub async fn start(&self, config: GatewayConfig, app: tauri::AppHandle) -> Result<(), String> {
        super::config_validation::validate(&config)?;
        if !config.enabled {
            return Err("Gateway désactivé".to_string());
        }
        audit::configure(&config.audit);
        let mut state = self.state.write().await;
        state.cancel.cancel();
        state.cancel = CancellationToken::new();
        state.channels.clear();
        state.adapters.clear();
        state.config = config.clone();
        state.limits = Arc::new(Mutex::new(GatewayRateLimiters::new(&config.rate_limits)));

        let (tx, mut rx) = mpsc::channel::<InboundMessage>(256);

        self.start_channel_accounts(&mut state, "telegram", &config.channels.telegram, &tx, &app);
        self.start_channel_accounts(&mut state, "slack", &config.channels.slack, &tx, &app);
        self.start_channel_accounts(&mut state, "discord", &config.channels.discord, &tx, &app);

        let bridge = Arc::new(GatewayAgentBridge::new(
            state.limits.clone(),
            config.max_sessions as usize,
        ));
        let state_ref = self.state.clone();
        let app_ref = app.clone();

        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                let adapter = {
                    let s = state_ref.read().await;
                    s.adapters.get(&msg.channel_key).cloned()
                };
                if let Some(adapter) = adapter {
                    let b = Arc::clone(&bridge);
                    let a = app_ref.clone();
                    tokio::spawn(async move {
                        let _ = b.process(msg, adapter, a).await;
                    });
                }
            }
        });

        let _ = app.emit("gateway-status-changed", build_health(&state));
        Ok(())
    }

    fn start_channel_accounts(
        &self,
        state: &mut GatewayState,
        channel_id: &str,
        accounts: &[ChannelAccountConfig],
        tx: &mpsc::Sender<InboundMessage>,
        app: &tauri::AppHandle,
    ) {
        for acc in accounts {
            if !acc.enabled {
                continue;
            }
            let key = ChannelKey::new(channel_id, &acc.account_id);
            let child_cancel = state.cancel.child_token();
            let adapter: Arc<dyn ChannelAdapter> = match channel_id {
                "telegram" => Arc::new(TelegramAdapter::new()),
                "slack" => Arc::new(super::channels::slack::SlackAdapter::new()),
                "discord" => Arc::new(super::channels::discord::DiscordAdapter::new()),
                _ => continue,
            };
            if let Err(message) = validate_account(channel_id, acc) {
                let error =
                    if service_audit::invalid_account_config(channel_id, &acc.account_id, &message)
                        .is_err()
                    {
                        "auditUnavailable"
                    } else {
                        "invalidConfig"
                    };
                state.channels.insert(
                    key.clone(),
                    ChannelEntry {
                        status: ChannelStatus::Error,
                        cancel: child_cancel,
                        error: Some(error.to_string()),
                    },
                );
                emit_channel_status(app, &key, ChannelStatus::Error, Some(error));
                continue;
            }
            state.adapters.insert(key.clone(), Arc::clone(&adapter));
            let ctx = ChannelContext {
                key: key.clone(),
                config: acc.clone(),
                cancel: child_cancel.clone(),
            };
            state.channels.insert(
                key.clone(),
                ChannelEntry {
                    status: ChannelStatus::Starting,
                    cancel: child_cancel,
                    error: None,
                },
            );
            let sender = tx.clone();
            let state_arc = self.state.clone();
            let key_clone = key;
            let app_clone = app.clone();
            tokio::spawn(async move {
                run_supervised_channel(adapter, ctx, sender, state_arc, key_clone, app_clone).await;
            });
        }
    }

    pub async fn stop(&self) {
        let mut state = self.state.write().await;
        state.cancel.cancel();
        for (key, entry) in state.channels.iter_mut() {
            entry.cancel.cancel();
            entry.status = ChannelStatus::Stopping;
            if service_audit::channel_stopped(key, None, None).is_err() {
                entry.error = Some("auditUnavailable".to_string());
            }
        }
        state.adapters.clear();
    }

    pub async fn health(&self) -> GatewayHealth {
        let s = self.state.read().await;
        build_health(&s)
    }
    pub async fn is_enabled(&self) -> bool {
        let s = self.state.read().await;
        s.config.enabled && !s.cancel.is_cancelled()
    }
    pub async fn config(&self) -> GatewayConfig {
        self.state.read().await.config.clone()
    }
    pub async fn update_config(&self, config: GatewayConfig) {
        audit::configure(&config.audit);
        self.state.write().await.config = config;
    }
}

#[cfg(test)]
#[path = "service_tests.rs"]
mod tests;
