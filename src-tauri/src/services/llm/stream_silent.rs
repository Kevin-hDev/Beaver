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
    let transport = super::stream_dispatch::resolve_transport(
        provider_id,
        model,
        super::stream_dispatch::InvocationKind::Silent,
        purpose,
    )
    .await
    .map_err(super::stream_dispatch::RouteSelectionError::code)?;
    let request_timeout = crate::services::compress::timeouts::compression_request_timeout();
    let idle_timeout = crate::services::compress::timeouts::compression_idle_timeout();
    let generated_request_id = uuid::Uuid::new_v4().to_string();
    let request_id = request_id.unwrap_or(&generated_request_id);
    let mut measurement = super::stream_metrics::start(
        &transport,
        provider_id,
        model,
        Some(session_id),
        request_id,
        None,
        1,
        crate::services::provider_usage::UsageWorkload::Compression,
        fast_mode,
    );
    let result = match transport.client {
        super::stream_dispatch::ClientKind::Anthropic => {
            let config = request_config(
                provider_id,
                fast_mode,
                model,
                messages,
                Some(max_tokens),
                purpose,
                Some(session_id),
            );
            super::anthropic::collect_silent(&config, cancel, measurement.as_mut()).await
        }
        super::stream_dispatch::ClientKind::Codex => {
            crate::services::codex_client::stream::collect_chat_silent_for_compression(
                model,
                messages,
                &[],
                None,
                fast_mode,
                Some(max_tokens),
                Some(session_id),
                cancel,
                measurement.as_mut(),
            )
            .await
        }
        super::stream_dispatch::ClientKind::Responses => {
            let config = request_config(
                provider_id,
                fast_mode,
                model,
                messages,
                Some(max_tokens),
                purpose,
                Some(session_id),
            );
            super::openai_responses::collect_silent(&config, cancel, measurement.as_mut()).await
        }
        super::stream_dispatch::ClientKind::ChatCompletions => {
            let cfg = request_config(
                provider_id,
                fast_mode,
                model,
                messages,
                Some(max_tokens),
                purpose,
                Some(session_id),
            );
            match post_chat_request_with_timeout_measured(
                &cfg,
                request_timeout,
                measurement.as_mut(),
                None,
            )
            .await
            {
                Ok(resp) => {
                    super::stream_silent_consume::consume_silent(
                        resp,
                        cancel,
                        idle_timeout,
                        transport.usage_context(model),
                        transport.fragment_mode,
                        transport.error_policy,
                        measurement.as_mut(),
                    )
                    .await
                }
                Err(error) => Err(error.to_string()),
            }
        }
        super::stream_dispatch::ClientKind::OllamaLocal => {
            let compression = crate::services::agent_local::ollama_collect::collect_chat_with_timeout_and_limit_global(
                model,
                messages.to_vec(),
                request_timeout,
                Some(max_tokens),
            );
            tokio::select! {
                _ = cancel.cancelled() => Err("Annulé".to_string()),
                result = compression => result.map(|(content, eval_count)| StreamResult {
                    content,
                    eval_count: Some(eval_count),
                    ..StreamResult::default()
                }),
            }
        }
        super::stream_dispatch::ClientKind::XaiOauth(_) => {
            Err("provider_configuration_invalid".to_string())
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
    let reasoning_mode = internal_reasoning_mode(provider_id, model);
    let fast_mode = reasoning_mode
        .map(|_| super::fast_mode::standard_for_internal(provider_id))
        .unwrap_or(fast_mode);
    RequestConfig {
        provider_id,
        fast_mode,
        model,
        messages,
        tools: &[],
        think: reasoning_mode.is_some(),
        reasoning_mode,
        max_tokens,
        purpose,
        session_id,
        tool_result_previews: None,
        continuation_target: None,
    }
}

fn internal_reasoning_mode(provider_id: &str, model: &str) -> Option<&'static str> {
    // Independent summaries use the lightest documented effort, never the
    // user's conversational selection or a costly remote default.
    match provider_id {
        "google" if model == "gemini-3.8-flash" => Some("low"),
        "zai" if model == "glm-5.3-flash" => Some("low"),
        "openai" if model == "gpt-6-astra" => Some("low"),
        _ => None,
    }
}

#[cfg(test)]
#[path = "stream_silent_request_tests.rs"]
mod stream_silent_request_tests;
