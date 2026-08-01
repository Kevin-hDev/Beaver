use std::time::Duration;

pub(super) const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
pub(super) const STREAM_STALL_TIMEOUT: Duration = Duration::from_secs(10 * 60);
pub(super) const MAX_STREAM_TEXT_BYTES: usize = 32 * 1024 * 1024;
