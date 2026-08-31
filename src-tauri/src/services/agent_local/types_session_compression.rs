use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionCompressionProfileSelection {
    pub profile_id: String,
    pub global_selection_revision: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AutomaticCompressionGuard {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_attempt: Option<AutomaticCompressionAttempt>,
    pub consecutive_failures: u8,
    pub suspended: bool,
}

impl AutomaticCompressionGuard {
    pub fn is_empty(&self) -> bool {
        self.last_attempt.is_none() && self.consecutive_failures == 0 && !self.suspended
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutomaticCompressionAttempt {
    pub top_level_turn_id: String,
    pub last_message_id: String,
    pub message_count: u16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_checkpoint_message_id: Option<String>,
    pub provider_id: String,
    pub model_id: String,
    pub context_window: u64,
    pub profile_id: String,
    pub profile_revision: u64,
    pub global_selection_revision: u64,
}
