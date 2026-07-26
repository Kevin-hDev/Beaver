use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{Mutex, RwLock};
use tokio_util::sync::CancellationToken;

use super::channels::ChannelAdapter;
use super::security::rate_state::GatewayRateLimiters;
use super::types::{ChannelHealthEntry, ChannelKey, ChannelStatus, GatewayHealth};
use crate::models::GatewayConfig;

pub(crate) struct ChannelEntry {
    pub(crate) status: ChannelStatus,
    pub(crate) cancel: CancellationToken,
    pub(crate) error: Option<String>,
}

pub struct GatewayState {
    pub(crate) channels: HashMap<ChannelKey, ChannelEntry>,
    pub(crate) adapters: HashMap<ChannelKey, Arc<dyn ChannelAdapter>>,
    pub(crate) config: GatewayConfig,
    pub(crate) cancel: CancellationToken,
    pub(crate) limits: Arc<Mutex<GatewayRateLimiters>>,
}

impl GatewayState {
    pub(crate) fn new() -> Self {
        Self {
            channels: HashMap::new(),
            adapters: HashMap::new(),
            config: GatewayConfig::default(),
            cancel: CancellationToken::new(),
            limits: Arc::new(Mutex::new(GatewayRateLimiters::new(
                &GatewayConfig::default().rate_limits,
            ))),
        }
    }
}

pub(crate) fn shared_state() -> Arc<RwLock<GatewayState>> {
    Arc::new(RwLock::new(GatewayState::new()))
}

pub(crate) fn build_health(state: &GatewayState) -> GatewayHealth {
    let channels = state
        .channels
        .iter()
        .map(|(key, entry)| ChannelHealthEntry {
            channel_id: key.channel_id.clone(),
            account_id: key.account_id.clone(),
            status: entry.status,
            error: entry.error.clone(),
        })
        .collect();
    GatewayHealth {
        running: !state.cancel.is_cancelled(),
        channels,
    }
}
