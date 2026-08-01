use super::stream_http::{
    post_chat_request_measured, post_chat_request_with_timeout_measured, RequestConfig,
};
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::agent_local::types_ollama::StreamResult;
use crate::services::llm::request_purpose::RequestPurpose;
use tokio_util::sync::CancellationToken;

pub async fn collect_chat_silent(
    provider_id: &str,
    model: &str,
    messages: &[ChatMessage],
    purpose: RequestPurpose,
    session_id: Option<&str>,
    cancel: CancellationToken,
) -> Result<StreamResult, String> {
    let request_id = uuid::Uuid::new_v4().to_string();
    let mut measurement = super::stream_metrics::start(
        provider_id,
        model,
        session_id,
        &request_id,
        None,
        1,
        crate::services::provider_usage::UsageWorkload::Primary,
    );
    let result = if provider_id == "codex-oauth" {
        crate::services::codex_client::stream::collect_chat_silent(
            model,
            messages,
            &[],
            None,
            None,
            session_id,
            cancel,
        )
        .await
    } else {
        let cfg = request_config(provider_id, model, messages, None, purpose, session_id);
        match post_chat_request_measured(&cfg, measurement.as_mut()).await {
            Ok(resp) => {
                super::stream_silent_consume::consume_silent(
                    resp,
                    cancel,
                    super::timeouts::idle_timeout_for(provider_id),
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

pub async fn collect_chat_silent_for_compression(
    provider_id: &str,
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
        )
        .await
    } else {
        let cfg = request_config(
            provider_id,
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
    model: &'a str,
    messages: &'a [ChatMessage],
    max_tokens: Option<u32>,
    purpose: RequestPurpose,
    session_id: Option<&'a str>,
) -> RequestConfig<'a> {
    RequestConfig {
        provider_id,
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
