use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::{mpsc, RwLock};
use tokio_tungstenite::tungstenite::Message as WsMessage;
use zeroize::Zeroizing;

use super::slack_types::*;
use super::websocket_limits::bounded_websocket_config;
use super::{
    capabilities::ChannelCapabilities, ChannelAdapter, ChannelContext, GatewayError, GatewayResult,
    InboundMessage, OutboundMessage,
};
use crate::services::gateway::tokens;
use crate::services::secure_http::{read_json_bounded, AuthenticatedClient, SLACK_BODY_LIMIT};

pub struct SlackAdapter {
    pub(super) client: AuthenticatedClient,
    pub(super) state: Arc<RwLock<SlackState>>,
}

pub(super) struct SlackState {
    pub(super) app_token: Option<Zeroizing<String>>,
    pub(super) bot_token: Option<Zeroizing<String>>,
    pub(super) bot_user_id: String,
}

impl SlackAdapter {
    pub fn new() -> Self {
        Self {
            client: AuthenticatedClient::new(Duration::from_secs(30)).expect("http client"),
            state: Arc::new(RwLock::new(SlackState {
                app_token: None,
                bot_token: None,
                bot_user_id: String::new(),
            })),
        }
    }
}

#[async_trait]
impl ChannelAdapter for SlackAdapter {
    fn capabilities(&self) -> ChannelCapabilities {
        ChannelCapabilities::slack()
    }

    async fn validate_config(
        &self,
        cfg: &crate::models::ChannelAccountConfig,
    ) -> GatewayResult<()> {
        if !tokens::has("slack", &cfg.account_id, "bot").unwrap_or(false) {
            return Err(GatewayError::auth("token bot Slack non configuré"));
        }
        if !tokens::has("slack", &cfg.account_id, "app").unwrap_or(false) {
            return Err(GatewayError::auth("token app Slack non configuré"));
        }
        Ok(())
    }

    async fn start(
        &self,
        ctx: ChannelContext,
        sender: mpsc::Sender<InboundMessage>,
    ) -> GatewayResult<tokio::task::JoinHandle<()>> {
        self.load_tokens(&format!("gateway.slack.{}", ctx.key.account_id))
            .await?;
        let state = self.state.clone();
        let client = self.client.clone();
        let cancel = ctx.cancel;
        let key = ctx.key;
        let require_mention = ctx.config.require_mention;
        let bot_user_id = self.state.read().await.bot_user_id.clone();

        Ok(tokio::spawn(async move {
            loop {
                if cancel.is_cancelled() {
                    break;
                }
                let app_token = {
                    let s = state.read().await;
                    match &s.app_token {
                        Some(t) => t.clone(),
                        None => break,
                    }
                };
                let ws_url = match Self::get_ws_url(&client, &app_token).await {
                    Ok(u) => u,
                    Err(_) => {
                        tokio::time::sleep(Duration::from_secs(10)).await;
                        continue;
                    }
                };
                let ws = match tokio_tungstenite::connect_async_with_config(
                    ws_url.as_str(),
                    Some(bounded_websocket_config(SLACK_BODY_LIMIT)),
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
                let (mut sink, mut stream) = ws.split();
                loop {
                    tokio::select! {
                        _ = cancel.cancelled() => break,
                        msg = stream.next() => {
                            let Some(Ok(WsMessage::Text(txt))) = msg else { break; };
                            let Ok(sm) = serde_json::from_str::<SlackSocketMessage>(&txt) else { continue; };
                            if let Some(eid) = &sm.envelope_id {
                                let ack = serde_json::to_string(&SlackAck { envelope_id: eid.clone() }).unwrap_or_default();
                                let _ = sink.send(WsMessage::Text(ack.into())).await;
                            }
                            if sm.msg_type != "events_api" {
                                continue;
                            }
                            if let Some(payload) = &sm.payload {
                                if let Some(evt) = &payload.event {
                                    if let Some(inbound) = Self::to_inbound(
                                        evt,
                                        &key,
                                        require_mention,
                                        &bot_user_id,
                                    ) {
                                        let _ = sender.send(inbound).await;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }))
    }

    async fn send(&self, msg: OutboundMessage) -> GatewayResult<()> {
        let token = {
            let s = self.state.read().await;
            s.bot_token
                .clone()
                .ok_or_else(|| GatewayError::auth("pas de token bot"))?
        };
        let body = Self::post_body(&msg);
        let request = self
            .client
            .post("https://slack.com/api/chat.postMessage")
            .bearer_auth(token.as_str())
            .json(&body);
        let response = self
            .client
            .send(request)
            .await
            .map_err(|_| GatewayError::network("envoi Slack impossible"))?;
        let resp: SlackPostResponse = read_json_bounded(response, SLACK_BODY_LIMIT)
            .await
            .map_err(|_| GatewayError::network("réponse Slack invalide"))?;

        if resp.ok && resp.ts.is_some() {
            Ok(())
        } else {
            Err(GatewayError::network("envoi Slack refusé"))
        }
    }
}
