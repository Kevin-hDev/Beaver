use std::time::Duration;

pub(crate) const LLM_IDLE_TIMEOUT_SECS: u64 = 180;
pub(crate) const LLM_REQUEST_TIMEOUT_SECS: u64 = 180;
const LLM_CONNECT_TIMEOUT_SECS: u64 = 10;

pub(crate) fn idle_timeout_for(provider_id: &str) -> Duration {
    duration_for(provider_id, LLM_IDLE_TIMEOUT_SECS)
}

pub(crate) fn request_timeout_for(provider_id: &str) -> Duration {
    duration_for(provider_id, LLM_REQUEST_TIMEOUT_SECS)
}

pub(crate) const fn connect_timeout() -> Duration {
    Duration::from_secs(LLM_CONNECT_TIMEOUT_SECS)
}

fn duration_for(provider_id: &str, default_seconds: u64) -> Duration {
    Duration::from_secs(
        crate::services::llm::route_profile::request_timeout_seconds(provider_id, default_seconds),
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn deepseek_keeps_its_documented_wait_window() {
        assert_eq!(super::request_timeout_for("deepseek").as_secs(), 600);
        assert_eq!(super::idle_timeout_for("deepseek").as_secs(), 600);
        assert_eq!(super::request_timeout_for("openai").as_secs(), 180);
        assert_eq!(super::connect_timeout().as_secs(), 10);
    }
}
