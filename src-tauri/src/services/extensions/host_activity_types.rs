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

impl ExtensionEventActivity {
    const MAX_SAFE_JAVASCRIPT_INTEGER: u64 = 9_007_199_254_740_991;

    pub(super) fn is_bounded(&self) -> bool {
        [
            self.queued,
            self.delivered,
            self.dropped,
            self.timed_out,
            self.active_handlers,
        ]
        .into_iter()
        .all(|value| value <= Self::MAX_SAFE_JAVASCRIPT_INTEGER)
    }

    pub(super) fn clamp(&mut self) {
        self.queued = self.queued.min(Self::MAX_SAFE_JAVASCRIPT_INTEGER);
        self.delivered = self.delivered.min(Self::MAX_SAFE_JAVASCRIPT_INTEGER);
        self.dropped = self.dropped.min(Self::MAX_SAFE_JAVASCRIPT_INTEGER);
        self.timed_out = self.timed_out.min(Self::MAX_SAFE_JAVASCRIPT_INTEGER);
        self.active_handlers = self.active_handlers.min(Self::MAX_SAFE_JAVASCRIPT_INTEGER);
    }
}
