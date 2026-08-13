use std::sync::Arc;
use std::time::Duration;

use futures_util::{Sink, SinkExt};
use tokio::sync::{mpsc, RwLock};
use tokio_tungstenite::tungstenite::Message as WsMessage;
use zeroize::Zeroizing;

use super::backpressure::{try_enqueue, EnqueueOutcome};
use super::discord::{DiscordAdapter, DiscordState};
use super::discord_gateway::{build_identify, HeartbeatSequence, SecretTextPayload};
use super::discord_types::{DiscordMessage, GatewayHello, GatewayPayload, ReadyEvent};
use super::InboundMessage;
use crate::services::gateway::refusal_audit::RefusalAudit;
use crate::services::gateway::types::ChannelKey;

pub(super) struct GatewayMessageContext<'a> {
    pub(super) state: &'a Arc<RwLock<DiscordState>>,
    pub(super) key: &'a ChannelKey,
    pub(super) require_mention: bool,
    pub(super) token: &'a str,
    pub(super) sender: &'a mpsc::Sender<InboundMessage>,
    pub(super) refusal_audit: &'a RefusalAudit,
    pub(super) sequence: &'a HeartbeatSequence,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[must_use = "the caller must distinguish reconnection from consumer shutdown"]
pub(super) enum DiscordEventOutcome {
    Continue,
    Reconnect,
    ConsumerClosed,
}

pub(super) async fn handle_gateway_message<S>(
    text: &str,
    context: &GatewayMessageContext<'_>,
    heartbeat: &mut Option<tokio::time::Interval>,
    sink: &mut S,
) -> DiscordEventOutcome
where
    S: Sink<WsMessage> + Unpin,
{
    let Ok(payload) = serde_json::from_str::<GatewayPayload>(text) else {
        return DiscordEventOutcome::Continue;
    };
    if let Some(value) = payload.s {
        context.sequence.update(value).await;
    }
    match payload.op {
        10 => handle_hello(&payload, context, heartbeat, sink).await,
        0 if payload.t.as_deref() == Some("READY") => {
            if let Some(data) = &payload.d {
                if let Ok(ready) = serde_json::from_value::<ReadyEvent>(data.clone()) {
                    context.state.write().await.bot_user_id = ready.user.id;
                }
            }
            DiscordEventOutcome::Continue
        }
        0 if payload.t.as_deref() == Some("MESSAGE_CREATE") => {
            handle_message_create(payload, context).await
        }
        _ => DiscordEventOutcome::Continue,
    }
}

async fn handle_hello<S>(
    payload: &GatewayPayload,
    context: &GatewayMessageContext<'_>,
    heartbeat: &mut Option<tokio::time::Interval>,
    sink: &mut S,
) -> DiscordEventOutcome
where
    S: Sink<WsMessage> + Unpin,
{
    if let Some(data) = &payload.d {
        if let Ok(hello) = serde_json::from_value::<GatewayHello>(data.clone()) {
            let every = Duration::from_millis(hello.heartbeat_interval);
            *heartbeat = Some(tokio::time::interval_at(
                tokio::time::Instant::now() + every,
                every,
            ));
        }
    }
    let json =
        Zeroizing::new(serde_json::to_string(&build_identify(context.token)).unwrap_or_default());
    let mut payload = SecretTextPayload::new(json.as_str());
    let Ok(message) = payload.message() else {
        let _ = payload.zeroize_after_send();
        return DiscordEventOutcome::Reconnect;
    };
    let sent = sink.send(message).await.is_ok();
    if !payload.zeroize_after_send() {
        // The network library may still own its frame; never log the payload.
        ::log::warn!("[gateway] nettoyage du payload Discord incomplet");
    }
    if sent {
        DiscordEventOutcome::Continue
    } else {
        DiscordEventOutcome::Reconnect
    }
}

async fn handle_message_create(
    payload: GatewayPayload,
    context: &GatewayMessageContext<'_>,
) -> DiscordEventOutcome {
    let Some(data) = payload.d else {
        return DiscordEventOutcome::Continue;
    };
    let Ok(message) = serde_json::from_value::<DiscordMessage>(data) else {
        return DiscordEventOutcome::Continue;
    };
    let bot_id = context.state.read().await.bot_user_id.clone();
    let Some(inbound) =
        DiscordAdapter::to_inbound(&message, context.key, context.require_mention, &bot_id)
    else {
        return DiscordEventOutcome::Continue;
    };
    match try_enqueue(context.sender, inbound, context.key, context.refusal_audit) {
        EnqueueOutcome::Closed => DiscordEventOutcome::ConsumerClosed,
        EnqueueOutcome::Enqueued | EnqueueOutcome::Full => DiscordEventOutcome::Continue,
    }
}
