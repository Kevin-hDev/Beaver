use super::stream_http::RequestConfig;

pub(super) fn requested_effort(config: &RequestConfig<'_>) -> Option<&'static str> {
    if config.provider_id == "xai" {
        return super::providers::xai::reasoning_effort(config.model, config.reasoning_mode);
    }
    if config.think || config.reasoning_mode == Some("off") {
        return crate::services::reasoning::openai_effort(config.reasoning_mode);
    }
    None
}
