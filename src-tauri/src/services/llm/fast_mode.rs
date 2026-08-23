#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FastModeRequest {
    Unsupported,
    Standard,
    Fast,
}

impl FastModeRequest {
    pub const fn for_api(supported: bool, enabled: bool) -> Self {
        if supported && enabled {
            Self::Fast
        } else {
            Self::Standard
        }
    }

    pub const fn for_codex(supported: bool, enabled: bool) -> Self {
        match (supported, enabled) {
            (false, _) => Self::Unsupported,
            (true, false) => Self::Standard,
            (true, true) => Self::Fast,
        }
    }

    pub const fn api_value(self) -> Option<&'static str> {
        match self {
            Self::Unsupported => None,
            Self::Standard => Some("default"),
            Self::Fast => Some("fast"),
        }
    }

    #[allow(
        dead_code,
        reason = "Task 4 transports the already-defined Codex mapping"
    )]
    pub const fn codex_value(self) -> Option<&'static str> {
        match self {
            Self::Unsupported | Self::Standard => None,
            Self::Fast => Some("priority"),
        }
    }

    #[allow(
        dead_code,
        reason = "Task 5 records the captured request in diagnostics"
    )]
    pub const fn fast_requested(self) -> bool {
        matches!(self, Self::Fast)
    }
}

pub async fn for_session(
    session_id: &str,
    provider_id: &str,
    model: &str,
) -> Result<FastModeRequest, String> {
    if provider_id != super::providers::openai::PROVIDER_ID
        && provider_id != crate::services::codex_client::PROVIDER_ID
    {
        return Ok(FastModeRequest::Unsupported);
    }
    let enabled = crate::services::agent_local::session_store::get(session_id)
        .await?
        .fast_mode_enabled;
    if provider_id == super::providers::openai::PROVIDER_ID {
        // `default` neutralise aussi le tier Fast configuré à distance sur le projet OpenAI.
        return Ok(FastModeRequest::for_api(
            super::provider_model_lookup::supports_fast_mode(provider_id, model),
            enabled,
        ));
    }
    let supported = crate::services::codex_client::model_catalog::supports_fast_mode(model)
        .await
        .unwrap_or(false);
    // Un catalogue OAuth indisponible ne peut jamais autoriser Fast.
    Ok(FastModeRequest::for_codex(supported, enabled))
}

pub(crate) fn standard_for_internal(provider_id: &str) -> FastModeRequest {
    // Les résumés et diagnostics internes ne doivent jamais hériter d'une préférence de session.
    if provider_id == super::providers::openai::PROVIDER_ID
        || provider_id == crate::services::codex_client::PROVIDER_ID
    {
        FastModeRequest::Standard
    } else {
        FastModeRequest::Unsupported
    }
}

#[cfg(test)]
mod tests {
    use super::{standard_for_internal, FastModeRequest};

    fn api_payload(fast_mode: FastModeRequest, model: &str) -> serde_json::Value {
        let cfg = super::super::stream_http::RequestConfig {
            provider_id: "openai",
            model,
            messages: &[],
            tools: &[],
            think: false,
            reasoning_mode: None,
            max_tokens: None,
            purpose: super::super::request_purpose::RequestPurpose::ManualChat,
            session_id: None,
            fast_mode,
        };
        let route = super::super::route::resolve("openai").expect("OpenAI route");
        super::super::stream_http_payload::build_chat_payload(&cfg, &route, None)
    }

    #[test]
    fn api_decision_always_neutralizes_inactive_fast() {
        assert_eq!(
            FastModeRequest::for_api(false, false),
            FastModeRequest::Standard
        );
        assert_eq!(
            FastModeRequest::for_api(false, true),
            FastModeRequest::Standard
        );
        assert_eq!(
            FastModeRequest::for_api(true, false),
            FastModeRequest::Standard
        );
        assert_eq!(FastModeRequest::for_api(true, true), FastModeRequest::Fast);
    }

    #[test]
    fn codex_decision_omits_unavailable_or_inactive_fast() {
        assert_eq!(
            FastModeRequest::for_codex(false, true),
            FastModeRequest::Unsupported
        );
        assert_eq!(
            FastModeRequest::for_codex(true, false),
            FastModeRequest::Standard
        );
        assert_eq!(
            FastModeRequest::for_codex(true, true),
            FastModeRequest::Fast
        );
    }

    #[test]
    fn transport_values_are_closed_and_provider_specific() {
        assert_eq!(FastModeRequest::Standard.api_value(), Some("default"));
        assert_eq!(FastModeRequest::Fast.api_value(), Some("fast"));
        assert_eq!(FastModeRequest::Standard.codex_value(), None);
        assert_eq!(FastModeRequest::Fast.codex_value(), Some("priority"));
    }

    #[test]
    fn internal_requests_never_enable_fast() {
        let api = standard_for_internal("openai");
        assert_eq!(api, FastModeRequest::Standard);
        assert_eq!(api.api_value(), Some("default"));
        assert_eq!(standard_for_internal("codex-oauth").codex_value(), None);
        assert_eq!(standard_for_internal("openrouter").api_value(), None);
    }

    #[tokio::test]
    async fn unrelated_providers_do_not_read_session_state() {
        assert_eq!(
            super::for_session("missing-session", "openrouter", "openai/gpt-5.6-luna")
                .await
                .expect("unsupported providers bypass session storage"),
            FastModeRequest::Unsupported
        );
        assert!(
            super::for_session("missing-session", "openai", "gpt-5.6-luna")
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn generation_capture_survives_session_changes_until_the_next_request() {
        let session =
            crate::services::agent_local::session_store::create_with_project_and_fast_mode(
                "Fast capture",
                "gpt-5.6-luna",
                "openai",
                None,
                true,
            )
            .await
            .expect("create session");
        let captured = super::for_session(&session.id, "openai", "gpt-5.6-luna")
            .await
            .expect("capture fast mode");

        crate::services::agent_local::session_store::update_fast_mode(&session.id, false)
            .await
            .expect("disable next generation");
        let retry_payload = api_payload(captured, "gpt-5.6-luna");
        let compression_payload = api_payload(captured, "gpt-5.6-luna");
        let next = super::for_session(&session.id, "openai", "gpt-5.6-luna")
            .await
            .expect("capture next generation");
        let next_payload = api_payload(next, "gpt-5.6-luna");

        crate::services::agent_local::session_store::update_fast_mode(&session.id, true)
            .await
            .expect("restore preference");
        let unadvertised = super::for_session(&session.id, "openai", "unadvertised-model")
            .await
            .expect("capture unadvertised model");
        let unadvertised_payload = api_payload(unadvertised, "unadvertised-model");
        crate::services::agent_local::session_store::delete_one(&session.id)
            .await
            .expect("delete session");

        assert_eq!(retry_payload["service_tier"], "fast");
        assert_eq!(compression_payload["service_tier"], "fast");
        assert_eq!(next_payload["service_tier"], "default");
        assert_eq!(unadvertised_payload["service_tier"], "default");
    }
}
