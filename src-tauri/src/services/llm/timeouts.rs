use std::time::Duration;

pub(crate) const LLM_IDLE_TIMEOUT_SECS: u64 = 180;
pub(crate) const LLM_REQUEST_TIMEOUT_SECS: u64 = 180;

pub(crate) fn idle_timeout() -> Duration {
    Duration::from_secs(LLM_IDLE_TIMEOUT_SECS)
}

pub(crate) fn request_timeout() -> Duration {
    Duration::from_secs(LLM_REQUEST_TIMEOUT_SECS)
}
