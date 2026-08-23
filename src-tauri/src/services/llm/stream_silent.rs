#![expect(
    clippy::too_many_arguments,
    reason = "orchestration boundary keeps related runtime context explicit"
)]
use super::stream_http::{post_chat_request_with_timeout_measured, RequestConfig};
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::agent_local::types_ollama::StreamResult;
use crate::services::llm::request_purpose::RequestPurpose;
use tokio_util::sync::CancellationToken;

pub async fn collect_chat_silent_for_compression(
    provider_id: &str,
    fast_mode: super::fast_mode::FastModeRequest,
    model: &str,
    messages: &[ChatMessage],
    max_tokens: u32,
    purpose: RequestPurpose,
    session_id: &str,
    request_id: Option<&str>,
    cancel: CancellationToken,
) -> Result<StreamResult, String> {
    let request_timeout = crate::services::compress::timeouts::compression_request_timeout();
    let idle_timeout = crate::services::compress::timeouts::compression_idle_timeout();
    let generated_request_id = uuid::Uuid::new_v4().to_string();
    let request_id = request_id.unwrap_or(&generated_request_id);
    let mut measurement = super::stream_metrics::start(
        provider_id,
        model,
        Some(session_id),
        request_id,
        None,
        1,
        crate::services::provider_usage::UsageWorkload::Compression,
    );
    let result = if provider_id == "codex-oauth" {
        crate::services::codex_client::stream::collect_chat_silent_for_compression(
            model,
            messages,
            &[],
            None,
            Some(max_tokens),
            Some(session_id),
            cancel,
            measurement.as_mut(),
        )
        .await
    } else {
        let cfg = request_config(
            provider_id,
            fast_mode,
            model,
            messages,
            Some(max_tokens),
            purpose,
            Some(session_id),
        );
        match post_chat_request_with_timeout_measured(&cfg, request_timeout, measurement.as_mut())
            .await
        {
            Ok(resp) => {
                super::stream_silent_consume::consume_silent(
                    resp,
                    cancel,
                    idle_timeout,
                    crate::services::provider_usage::UsageContext::chat(
                        crate::services::llm::route::canonical_provider_id(provider_id),
                        model,
                    ),
                    measurement.as_mut(),
                )
                .await
            }
            Err(error) => Err(error.to_string()),
        }
    };
    super::stream_metrics::finish_silent(measurement, &result).await;
    result
}

fn request_config<'a>(
    provider_id: &'a str,
    fast_mode: super::fast_mode::FastModeRequest,
    model: &'a str,
    messages: &'a [ChatMessage],
    max_tokens: Option<u32>,
    purpose: RequestPurpose,
    session_id: Option<&'a str>,
) -> RequestConfig<'a> {
    RequestConfig {
        provider_id,
        fast_mode,
        model,
        messages,
        tools: &[],
        think: false,
        reasoning_mode: None,
        max_tokens,
        purpose,
        session_id,
    }
}

#[cfg(test)]
mod tests {
    use super::request_config;
    use crate::services::llm::fast_mode::{standard_for_internal, FastModeRequest};
    use crate::services::llm::request_purpose::RequestPurpose;

    #[test]
    fn silent_compression_reuses_the_generation_capture() {
        let route = crate::services::llm::route::resolve("openai").expect("OpenAI route");
        let cfg = request_config(
            "openai",
            FastModeRequest::Fast,
            "gpt-5.6-luna",
            &[],
            Some(1_024),
            RequestPurpose::ManualChat,
            None,
        );

        let payload = crate::services::llm::stream_http_payload::build_chat_payload(
            &cfg,
            &route,
            Some(1_024),
        );
        assert_eq!(payload["service_tier"], "fast");
    }

    #[test]
    fn silent_internal_requests_never_enable_fast() {
        for (provider, model, expected) in [
            ("openai", "gpt-5.6-luna", Some("default")),
            ("openrouter", "openai/gpt-5.6-luna", None),
        ] {
            let route = crate::services::llm::route::resolve(provider).expect("provider route");
            let cfg = request_config(
                provider,
                standard_for_internal(provider),
                model,
                &[],
                Some(1_024),
                RequestPurpose::ManualChat,
                None,
            );
            let payload = crate::services::llm::stream_http_payload::build_chat_payload(
                &cfg,
                &route,
                Some(1_024),
            );

            assert_eq!(
                payload.get("service_tier").and_then(|value| value.as_str()),
                expected
            );
        }
        assert_eq!(standard_for_internal("codex-oauth").codex_value(), None);
    }
}
