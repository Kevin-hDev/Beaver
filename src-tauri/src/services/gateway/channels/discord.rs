use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio_tungstenite::tungstenite::Message as WsMessage;
use zeroize::{Zeroize, Zeroizing};

use super::discord_gateway::{build_identify, heartbeat_loop, HeartbeatSequence};
use super::discord_types::*;
use super::websocket_limits::bounded_websocket_config;
use super::{
    capabilities::ChannelCapabilities, ChannelAdapter, ChannelContext, GatewayError, GatewayResult,
    InboundMessage, OutboundMessage,
};
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
    ) -> GatewayResult<tokio::task::JoinHandle<()>> {
        self.load_token(&ctx.key.vault_key()).await?;
        let state = self.state.clone();
        let cancel = ctx.cancel;
        let key = ctx.key;
        let require_mention = ctx.config.require_mention;

        Ok(tokio::spawn(async move {
            loop {
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
                        tokio::time::sleep(Duration::from_secs(5)).await;
                        continue;
                    }
                };
                let (sink, mut stream) = ws.split();
                let shared_sink = Arc::new(Mutex::new(sink));
                let sequence = HeartbeatSequence::new();

                while let Some(Ok(WsMessage::Text(txt))) = stream.next().await {
                    if cancel.is_cancelled() {
                        break;
                    }
                    let Ok(payload) = serde_json::from_str::<GatewayPayload>(&txt) else {
                        continue;
                    };
                    if let Some(s) = payload.s {
                        sequence.update(s).await;
                    }
                    match payload.op {
                        10 => {
                            if let Some(d) = &payload.d {
                                if let Ok(hello) = serde_json::from_value::<GatewayHello>(d.clone())
                                {
                                    let interval = Duration::from_millis(hello.heartbeat_interval);
                                    tokio::spawn(heartbeat_loop(
                                        shared_sink.clone(),
                                        cancel.clone(),
                                        interval,
                                        sequence.clone(),
                                    ));
                                }
                            }
                            let mut json = serde_json::to_string(&build_identify(token.as_str()))
                                .unwrap_or_default();
                            let send_result = shared_sink
                                .lock()
                                .await
                                .send(WsMessage::Text(json.as_str().into()))
                                .await;
                            json.zeroize();
                            let _ = send_result;
                        }
                        0 if payload.t.as_deref() == Some("READY") => {
                            if let Some(d) = &payload.d {
                                if let Ok(ready) = serde_json::from_value::<ReadyEvent>(d.clone()) {
                                    state.write().await.bot_user_id = ready.user.id;
                                }
                            }
                        }
                        0 if payload.t.as_deref() == Some("MESSAGE_CREATE") => {
                            if let Some(d) = payload.d {
                                if let Ok(msg) = serde_json::from_value::<DiscordMessage>(d) {
                                    let bot_id = state.read().await.bot_user_id.clone();
                                    if let Some(inbound) =
                                        Self::to_inbound(&msg, &key, require_mention, &bot_id)
                                    {
                                        let _ = sender.send(inbound).await;
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                if cancel.is_cancelled() {
                    break;
                }
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }))
    }

    async fn send(&self, msg: OutboundMessage) -> GatewayResult<()> {
        self.send_message(msg).await
    }
}
