use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use tokio::sync::{mpsc, RwLock};
use zeroize::Zeroizing;

use super::telegram_types::*;
use super::{
    capabilities::ChannelCapabilities, ChannelAdapter, ChannelContext, GatewayError, GatewayResult,
    InboundMessage, OutboundMessage,
};
use crate::services::gateway::tokens;
use crate::services::secure_http::{read_json_bounded, AuthenticatedClient, TELEGRAM_BODY_LIMIT};

pub struct TelegramAdapter {
    pub(super) client: AuthenticatedClient,
    pub(super) state: Arc<RwLock<TelegramState>>,
}

pub(super) struct TelegramState {
    pub(super) bot_token: Option<Zeroizing<String>>,
    pub(super) bot_username: String,
    pub(super) last_offset: i64,
}

impl TelegramAdapter {
    pub fn new() -> Self {
        Self {
            client: AuthenticatedClient::new(Duration::from_secs(35)).expect("http client"),
            state: Arc::new(RwLock::new(TelegramState {
                bot_token: None,
                bot_username: String::new(),
                last_offset: 0,
            })),
        }
    }

    pub fn api_url(token: &str, method: &str) -> Zeroizing<String> {
        Zeroizing::new(format!("https://api.telegram.org/bot{token}/{method}"))
    }
}

#[async_trait]
impl ChannelAdapter for TelegramAdapter {
    fn capabilities(&self) -> ChannelCapabilities {
        ChannelCapabilities::telegram()
    }

    async fn validate_config(
        &self,
        cfg: &crate::models::ChannelAccountConfig,
    ) -> GatewayResult<()> {
        if !tokens::has("telegram", &cfg.account_id, "default").unwrap_or(false) {
            return Err(GatewayError::auth("token Telegram non configuré"));
        }
        Ok(())
    }

    async fn start(
        &self,
        ctx: ChannelContext,
        sender: mpsc::Sender<InboundMessage>,
    ) -> GatewayResult<tokio::task::JoinHandle<()>> {
        self.load_token_and_identity(&ctx.key.vault_key()).await?;
        let client = self.client.clone();
        let state = self.state.clone();
        let cancel = ctx.cancel;
        let require_mention = ctx.config.require_mention;
        let channel_key = ctx.key;

        Ok(tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = cancel.cancelled() => break,
                    result = Self::poll_updates(&client, &state) => {
                        match result {
                            Ok(updates) => {
                                let bot_name = state.read().await.bot_username.clone();
                                for u in updates {
                                    if let Some(m) = Self::to_inbound(&u, &channel_key, require_mention, &bot_name) {
                                        let _ = sender.send(m).await;
                                    }
                                }
                            }
                            Err(e) if e.is_auth => break,
                            Err(_) => tokio::time::sleep(Duration::from_secs(5)).await,
                        }
                    }
                }
            }
        }))
    }

    async fn send(&self, msg: OutboundMessage) -> GatewayResult<()> {
        let state = self.state.read().await;
        let token = state
            .bot_token
            .as_ref()
            .ok_or_else(|| GatewayError::auth("pas de token"))?;
        let url = Self::api_url(token, "sendMessage");

        let body = Self::send_body(&msg);

        let request = self.client.post(url.as_str()).json(&body);
        let response = self
            .client
            .send(request)
            .await
            .map_err(|_| GatewayError::network("envoi Telegram impossible"))?;
        if !response.status().is_success() {
            return Err(GatewayError::network("envoi Telegram refusé"));
        }
        let resp: TgResponse<TgSentMessage> = read_json_bounded(response, TELEGRAM_BODY_LIMIT)
            .await
            .map_err(|_| GatewayError::network("réponse Telegram invalide"))?;

        if resp.ok && resp.result.is_some() {
            Ok(())
        } else {
            Err(GatewayError::network("envoi Telegram refusé"))
        }
    }
}
