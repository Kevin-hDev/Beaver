use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExtensionHostActivity {
    pub events: ExtensionEventActivity,
    pub active_interceptors: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExtensionEventActivity {
    pub queued: u64,
    pub delivered: u64,
    pub dropped: u64,
    pub timed_out: u64,
    pub active_handlers: u64,
}
