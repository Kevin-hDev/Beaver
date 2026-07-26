use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelCapabilities {
    pub supports_dm: bool,
    pub supports_groups: bool,
    pub supports_threads: bool,
    pub max_message_chars: usize,
    pub utf16_length: bool,
}

impl ChannelCapabilities {
    pub fn telegram() -> Self {
        Self {
            supports_dm: true,
            supports_groups: true,
            supports_threads: false,
            max_message_chars: 4096,
            utf16_length: true,
        }
    }

    pub fn slack() -> Self {
        Self {
            supports_dm: true,
            supports_groups: true,
            supports_threads: true,
            max_message_chars: 40_000,
            utf16_length: false,
        }
    }

    pub fn discord() -> Self {
        Self {
            supports_dm: true,
            supports_groups: true,
            supports_threads: true,
            max_message_chars: 2000,
            utf16_length: false,
        }
    }
}
