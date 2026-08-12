use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::{mpsc, RwLock};
use tokio_tungstenite::tungstenite::Message as WsMessage;
use zeroize::Zeroizing;

use super::backpressure::{try_enqueue, EnqueueOutcome};
use super::discord_gateway::{build_identify, HeartbeatSequence, SecretTextPayload};
use super::discord_types::*;
use super::websocket_limits::bounded_websocket_config;
use super::{
    capabilities::ChannelCapabilities, ChannelAdapter, ChannelContext, GatewayError, GatewayResult,
    InboundMessage, OutboundMessage,
};
use crate::services::gateway::reconnect_policy::ReconnectPolicy;
use crate::services::gateway::tokens;
use crate::services::secure_http::{AuthenticatedClient, DISCORD_BODY_LIMIT};

pub struct DiscordAdapter {
    pub(super) client: AuthenticatedClient,
    pub(super) state: Arc<RwLock<DiscordState>>,
}

pub(super) struct DiscordState {
    pub(super) bot_token: Option<Zeroizing<String>>,
    pub(super) bot_user_id: String,
}

impl DiscordAdapter {
    pub fn new() -> Self {
        Self {
            client: AuthenticatedClient::new(Duration::from_secs(30)).expect("http client"),
            state: Arc::new(RwLock::new(DiscordState {
                bot_token: None,
                bot_user_id: String::new(),
            })),
        }
    }
}

#[async_trait]
impl ChannelAdapter for DiscordAdapter {
    fn capabilities(&self) -> ChannelCapabilities {
        ChannelCapabilities::discord()
    }

    async fn validate_config(
        &self,
        cfg: &crate::models::ChannelAccountConfig,
    ) -> GatewayResult<()> {
        if !tokens::has("discord", &cfg.account_id, "default").unwrap_or(false) {
            return Err(GatewayError::auth("token Discord non configuré"));
        }
        Ok(())
    }

    async fn start(
        &self,
        ctx: ChannelContext,
        sender: mpsc::Sender<InboundMessage>,
    ) -> GatewayResult<super::ChannelRun> {
        self.load_token(&ctx.key.vault_key()).await?;
        let state = self.state.clone();
        let cancel = ctx.cancel;
        let key = ctx.key;
        let refusal_audit = ctx.refusal_audit;
        let require_mention = ctx.config.require_mention;

        Ok(Box::pin(async move {
            let mut reconnect = ReconnectPolicy::new();
            'gateway: loop {
                if cancel.is_cancelled() {
                    break;
                }
                let token = {
                    let s = state.read().await;
                    match &s.bot_token {
                        Some(t) => t.clone(),
                        None => break,
                    }
                };
                let ws = match tokio_tungstenite::connect_async_with_config(
                    GATEWAY_URL,
                    Some(bounded_websocket_config(DISCORD_BODY_LIMIT)),
                    false,
                )
                .await
                {
                    Ok((s, _)) => s,
                    Err(_) => {
                        if !reconnect.wait(&cancel).await {
                            break;
                        }
                        continue;
                    }
                };
                let connected_at = std::time::Instant::now();
                let (mut sink, mut stream) = ws.split();
                let sequence = HeartbeatSequence::new();
                let mut heartbeat: Option<tokio::time::Interval> = None;

                loop {
                    let next_heartbeat = async {
                        match &mut heartbeat {
                            Some(interval) => interval.tick().await,
                            None => std::future::pending().await,
                        }
                    };
                    tokio::select! {
                        _ = cancel.cancelled() => break 'gateway,
                        _ = next_heartbeat => {
                            let heartbeat = Heartbeat { op: 1, d: sequence.current().await };
                            let json = serde_json::to_string(&heartbeat).unwrap_or_default();
                            if sink.send(WsMessage::Text(json.into())).await.is_err() {
                                break;
                            }
                            continue;
                        }
                        message = stream.next() => {
                            let Some(Ok(WsMessage::Text(txt))) = message else { break; };
                            let message_context = GatewayMessageContext {
                                state: &state,
                                key: &key,
                                require_mention,
                                token: token.as_str(),
                                sender: &sender,
                                refusal_audit: &refusal_audit,
                                sequence: &sequence,
                            };
                            if !handle_gateway_message(
                                &txt,
                                &message_context,
                                &mut heartbeat,
                                &mut sink,
                            )
                            .await
                            {
                                break 'gateway;
                            }
                        }
                    }
                }
                if cancel.is_cancelled() {
                    break;
                }
                reconnect.record_connection(connected_at.elapsed());
                if !reconnect.wait(&cancel).await {
                    break;
                }
            }
        }))
    }

    async fn send(&self, msg: OutboundMessage) -> GatewayResult<()> {
        self.send_message(msg).await
    }
}

struct GatewayMessageContext<'a> {
    state: &'a Arc<RwLock<DiscordState>>,
    key: &'a crate::services::gateway::types::ChannelKey,
    require_mention: bool,
    token: &'a str,
    sender: &'a mpsc::Sender<InboundMessage>,
    refusal_audit: &'a crate::services::gateway::refusal_audit::RefusalAudit,
    sequence: &'a HeartbeatSequence,
}

async fn handle_gateway_message(
    text: &str,
    context: &GatewayMessageContext<'_>,
    heartbeat: &mut Option<tokio::time::Interval>,
    sink: &mut super::discord_gateway::WsSink,
) -> bool {
    let Ok(payload) = serde_json::from_str::<GatewayPayload>(text) else {
        return true;
    };
    if let Some(value) = payload.s {
        context.sequence.update(value).await;
    }
    match payload.op {
        10 => {
            if let Some(data) = &payload.d {
                if let Ok(hello) = serde_json::from_value::<GatewayHello>(data.clone()) {
                    let every = Duration::from_millis(hello.heartbeat_interval);
                    *heartbeat = Some(tokio::time::interval_at(
                        tokio::time::Instant::now() + every,
                        every,
                    ));
                }
            }
            let json = Zeroizing::new(
                serde_json::to_string(&build_identify(context.token)).unwrap_or_default(),
            );
            let mut payload = SecretTextPayload::new(json.as_str());
            let Ok(message) = payload.message() else {
                let _ = payload.zeroize_after_send();
                return false;
            };
            let sent = sink.send(message).await.is_ok();
            let zeroized = payload.zeroize_after_send();
            sent && zeroized
        }
        0 if payload.t.as_deref() == Some("READY") => {
            if let Some(data) = &payload.d {
                if let Ok(ready) = serde_json::from_value::<ReadyEvent>(data.clone()) {
                    context.state.write().await.bot_user_id = ready.user.id;
                }
            }
            true
        }
        0 if payload.t.as_deref() == Some("MESSAGE_CREATE") => {
            if let Some(data) = payload.d {
                if let Ok(message) = serde_json::from_value::<DiscordMessage>(data) {
                    let bot_id = context.state.read().await.bot_user_id.clone();
                    if let Some(inbound) = DiscordAdapter::to_inbound(
                        &message,
                        context.key,
                        context.require_mention,
                        &bot_id,
                    ) {
                        if try_enqueue(context.sender, inbound, context.key, context.refusal_audit)
                            == EnqueueOutcome::Closed
                        {
                            return false;
                        }
                    }
                }
            }
            true
        }
        _ => true,
    }
}
