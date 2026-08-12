use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{Mutex, RwLock};
use tokio_util::sync::CancellationToken;

use super::channels::ChannelAdapter;
use super::refusal_audit::RefusalCounter;
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
    pub(super) refused_messages: RefusalCounter,
}

impl GatewayState {
    pub(crate) fn new() -> Self {
        let cancel = CancellationToken::new();
        // Une configuration persistée n'est pas une exécution : seul start()
        // remplace ce jeton annulé par celui du run effectivement possédé.
        cancel.cancel();
        Self {
            channels: HashMap::new(),
            adapters: HashMap::new(),
            config: GatewayConfig::default(),
            cancel,
            limits: Arc::new(Mutex::new(GatewayRateLimiters::new(
                &GatewayConfig::default().rate_limits,
            ))),
            refused_messages: RefusalCounter::default(),
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
        running: state.config.enabled && !state.cancel.is_cancelled(),
        channels,
        refused_messages: state.refused_messages.total(),
    }
}

#[cfg(test)]
mod tests {
    use super::{build_health, GatewayState};
    use crate::services::gateway::refusal_audit::RefusalAudit;
    use crate::services::gateway::types::ChannelKey;

    #[test]
    fn gateway_health_exposes_refusals_without_a_persistent_writer() {
        let (audit, receiver) = RefusalAudit::channel();
        drop(receiver);
        let _ = audit.record_refusal(ChannelKey::new("discord", "main"), "gateway_busy");
        let mut state = GatewayState::new();
        state.refused_messages = audit.counter();

        assert_eq!(build_health(&state).refused_messages, 1);
    }
}
