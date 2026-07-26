use std::time::Duration;

use crate::services::secure_http::MAX_AUTHENTICATED_TIMEOUT;

pub fn compression_request_timeout() -> Duration {
    MAX_AUTHENTICATED_TIMEOUT
}

pub fn compression_idle_timeout() -> Duration {
    Duration::from_secs(600)
}
