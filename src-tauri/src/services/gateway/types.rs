use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayHealth {
    pub running: bool,
    pub channels: Vec<ChannelHealthEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelHealthEntry {
    pub channel_id: String,
    pub account_id: String,
    pub status: ChannelStatus,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChannelStatus {
    Off,
    Starting,
    Running,
    Error,
    Stopping,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChannelKey {
    pub channel_id: String,
    pub account_id: String,
}

impl ChannelKey {
    pub fn new(channel_id: impl Into<String>, account_id: impl Into<String>) -> Self {
        Self {
            channel_id: channel_id.into(),
            account_id: account_id.into(),
        }
    }

    pub fn vault_key(&self) -> String {
        format!("gateway.{}.{}", self.channel_id, self.account_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channel_key_vault_format() {
        let key = ChannelKey::new("telegram", "main-bot");
        assert_eq!(key.vault_key(), "gateway.telegram.main-bot");
    }

    #[test]
    fn channel_key_equality() {
        let a = ChannelKey::new("slack", "work");
        let b = ChannelKey::new("slack", "work");
        let c = ChannelKey::new("slack", "personal");
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}
