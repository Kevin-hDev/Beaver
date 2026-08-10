pub(in crate::services::browser) const CEF_SLOT_CAPACITY: usize = 64;
pub(super) const CEF_MARKER_MAX_BYTES: usize = 128;
pub(super) const CEF_NONCE_BYTES: usize = 32;
pub(super) const GATE_RECHECK: std::time::Duration = std::time::Duration::from_millis(1);
pub(super) const CEF_TRACKER_POLL: std::time::Duration = std::time::Duration::from_millis(10);
