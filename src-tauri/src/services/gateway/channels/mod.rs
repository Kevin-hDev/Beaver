mod bounded_vec;
pub mod capabilities;
pub mod discord;
pub mod discord_gateway;
mod discord_http;
mod discord_support;
pub mod discord_types;
pub mod slack;
mod slack_support;
pub mod slack_types;
pub mod telegram;
mod telegram_support;
pub mod telegram_types;
mod websocket_limits;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use super::types::ChannelKey;
use crate::models::ChannelAccountConfig;
use capabilities::ChannelCapabilities;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboundMessage {
    pub channel_key: ChannelKey,
    pub user_id: String,
    pub content: String,
    pub message_id: String,
    pub chat_id: String,
    pub thread_id: Option<String>,
    pub is_group: bool,
    pub mentions_bot: bool,
}

#[derive(Debug, Clone)]
pub struct OutboundMessage {
    pub chat_id: String,
    pub thread_id: Option<String>,
    pub content: String,
    pub reply_to: Option<String>,
}

#[derive(Clone)]
pub struct ChannelContext {
    pub key: ChannelKey,
    pub config: ChannelAccountConfig,
    pub cancel: CancellationToken,
}

pub type GatewayResult<T> = Result<T, GatewayError>;

#[derive(Debug)]
pub struct GatewayError {
    pub message: String,
    pub is_auth: bool,
}

impl std::fmt::Display for GatewayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl GatewayError {
    pub fn network(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
            is_auth: false,
        }
    }

    pub fn auth(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
            is_auth: true,
        }
    }
}

#[async_trait]
pub trait ChannelAdapter: Send + Sync {
    fn capabilities(&self) -> ChannelCapabilities;

    async fn validate_config(&self, cfg: &ChannelAccountConfig) -> GatewayResult<()>;

    async fn start(
        &self,
        ctx: ChannelContext,
        sender: tokio::sync::mpsc::Sender<InboundMessage>,
    ) -> GatewayResult<JoinHandle<()>>;

    async fn send(&self, msg: OutboundMessage) -> GatewayResult<()>;
}
