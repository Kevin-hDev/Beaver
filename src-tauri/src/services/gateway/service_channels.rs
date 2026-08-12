use std::sync::Arc;

use tokio::sync::{mpsc, RwLock};

use super::channels::telegram::TelegramAdapter;
use super::channels::{ChannelAdapter, ChannelContext, InboundMessage};
use super::service_runtime::{emit_channel_status, run_supervised_channel, validate_account};
use super::service_state::{ChannelEntry, GatewayState};
use super::types::{ChannelKey, ChannelStatus};
use super::work_supervision::GatewayWorkServices;
use crate::models::{ChannelAccountConfig, GatewayConfig};

pub(super) fn start_channel_accounts(
    state: &mut GatewayState,
    shared_state: &Arc<RwLock<GatewayState>>,
    config: &GatewayConfig,
    sender: &mpsc::Sender<InboundMessage>,
    app: &tauri::AppHandle,
    work: &GatewayWorkServices,
) -> Result<(), String> {
    let context = StartContext {
        shared_state,
        sender,
        app,
        work,
    };
    start_provider(state, "telegram", &config.channels.telegram, &context)?;
    start_provider(state, "slack", &config.channels.slack, &context)?;
    start_provider(state, "discord", &config.channels.discord, &context)
}

struct StartContext<'a> {
    shared_state: &'a Arc<RwLock<GatewayState>>,
    sender: &'a mpsc::Sender<InboundMessage>,
    app: &'a tauri::AppHandle,
    work: &'a GatewayWorkServices,
}

fn start_provider(
    state: &mut GatewayState,
    channel_id: &str,
    accounts: &[ChannelAccountConfig],
    start: &StartContext<'_>,
) -> Result<(), String> {
    for account in accounts.iter().filter(|account| account.enabled) {
        let key = ChannelKey::new(channel_id, &account.account_id);
        let channel_cancel = state.cancel.child_token();
        let adapter = adapter_for(channel_id);
        if let Err(message) = validate_account(channel_id, account) {
            record_invalid_account(state, start.app, key, channel_cancel, &message);
            continue;
        }
        state.adapters.insert(key.clone(), Arc::clone(&adapter));
        state.channels.insert(
            key.clone(),
            ChannelEntry {
                status: ChannelStatus::Starting,
                cancel: channel_cancel.clone(),
                error: None,
            },
        );
        let context = ChannelContext {
            key: key.clone(),
            config: account.clone(),
            cancel: channel_cancel,
        };
        let task_sender = start.sender.clone();
        let task_state = Arc::clone(start.shared_state);
        let task_app = start.app.clone();
        start
            .work
            .spawn_channel(move |work_cancel| async move {
                tokio::select! {
                    _ = work_cancel.cancelled() => {}
                    _ = run_supervised_channel(
                        adapter,
                        context,
                        task_sender,
                        task_state,
                        key,
                        task_app,
                    ) => {}
                }
            })
            .map_err(|error| error.public_code().to_string())?;
    }
    Ok(())
}

fn adapter_for(channel_id: &str) -> Arc<dyn ChannelAdapter> {
    match channel_id {
        "telegram" => Arc::new(TelegramAdapter::new()),
        "slack" => Arc::new(super::channels::slack::SlackAdapter::new()),
        "discord" => Arc::new(super::channels::discord::DiscordAdapter::new()),
        _ => unreachable!("validated gateway provider"),
    }
}

fn record_invalid_account(
    state: &mut GatewayState,
    app: &tauri::AppHandle,
    key: ChannelKey,
    cancel: tokio_util::sync::CancellationToken,
    message: &str,
) {
    let error =
        if super::service_audit::invalid_account_config(&key.channel_id, &key.account_id, message)
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
            cancel,
            error: Some(error.to_string()),
        },
    );
    emit_channel_status(app, &key, ChannelStatus::Error, Some(error));
}
