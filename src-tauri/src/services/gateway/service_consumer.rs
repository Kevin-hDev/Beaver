use std::sync::Arc;

use tokio::sync::{mpsc, RwLock};
use tokio_util::sync::CancellationToken;

use super::agent_bridge::GatewayAgentBridge;
use super::channels::InboundMessage;
use super::service_state::GatewayState;
use super::work_supervision::GatewayWorkServices;
use crate::services::work_registry::ServiceWorkCancellation;

pub(super) async fn consume_messages(
    mut receiver: mpsc::Receiver<InboundMessage>,
    state: Arc<RwLock<GatewayState>>,
    bridge: Arc<GatewayAgentBridge>,
    app: tauri::AppHandle,
    work: GatewayWorkServices,
    run_cancel: CancellationToken,
    consumer_cancel: ServiceWorkCancellation,
) {
    loop {
        let message = tokio::select! {
            _ = consumer_cancel.cancelled() => return,
            message = receiver.recv() => message,
        };
        let Some(message) = message else {
            return;
        };
        let adapter = state
            .read()
            .await
            .adapters
            .get(&message.channel_key)
            .cloned();
        let Some(adapter) = adapter else {
            continue;
        };
        let channel_key = message.channel_key.clone();
        let message_bridge = Arc::clone(&bridge);
        let message_app = app.clone();
        let message_run_cancel = run_cancel.clone();
        if let Err(error) = work.spawn_message(move |message_cancel| async move {
            tokio::select! {
                _ = message_cancel.cancelled() => {}
                _ = message_bridge.process(
                    message,
                    adapter,
                    message_app,
                    message_run_cancel,
                ) => {}
            }
        }) {
            let diagnostics = work.message_diagnostics();
            ::log::warn!(
                "[gateway] {} active={} saturation_refusals={} closing_refusals={}",
                error.public_code(),
                diagnostics.active,
                diagnostics.saturation_refusals,
                diagnostics.closing_refusals,
            );
            let _ = super::service_audit::work_refused(&channel_key, error.audit_code());
        }
    }
}
