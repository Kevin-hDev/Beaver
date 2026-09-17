#![expect(
    clippy::too_many_arguments,
    reason = "the transport boundary keeps attribution and budgets explicit"
)]
use super::stream_http::{post_chat_request_with_timeout_measured, RequestConfig};
use crate::services::agent_local::types_ollama::{ChatMessage, StreamResult};
use crate::services::provider_usage::UsageWorkload;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

pub(crate) async fn collect(
    connection_id: &str,
    fast_mode: super::fast_mode::FastModeRequest,
    model: &str,
    messages: &[ChatMessage],
    max_tokens: u32,
    purpose: super::request_purpose::RequestPurpose,
    session_id: &str,
    request_id: &str,
    cancel: CancellationToken,
    workload: UsageWorkload,
    request_timeout: Duration,
    idle_timeout: Duration,
    max_text_bytes: usize,
    reasoning_mode: Option<&str>,
) -> Result<StreamResult, String> {
    let transport = super::stream_dispatch::resolve_transport(
        connection_id,
        model,
        super::stream_dispatch::InvocationKind::Silent,
        purpose,
    )
    .await
    .map_err(super::stream_dispatch::RouteSelectionError::code)?;
    let mut measurement = super::stream_metrics::start(
        &transport,
        connection_id,
        model,
        Some(session_id),
        request_id,
        None,
        1,
        workload,
        fast_mode,
    );
    let result = match transport.client {
        super::stream_dispatch::ClientKind::Anthropic => {
            let config = request_config(
                connection_id,
                fast_mode,
                model,
                messages,
                max_tokens,
                purpose,
                session_id,
                reasoning_mode,
            );
            super::anthropic::collect_silent_bounded(
                &config,
                cancel,
                max_text_bytes,
                measurement.as_mut(),
            )
            .await
        }
        super::stream_dispatch::ClientKind::Codex => {
            crate::services::codex_client::stream::collect_chat_silent(
                model,
                messages,
                &[],
                reasoning_mode,
                fast_mode,
                Some(max_tokens),
                Some(session_id),
                cancel,
                request_timeout,
                idle_timeout,
                max_text_bytes,
                measurement.as_mut(),
            )
            .await
        }
        super::stream_dispatch::ClientKind::Responses => {
            let config = request_config(
                connection_id,
                fast_mode,
                model,
                messages,
                max_tokens,
                purpose,
                session_id,
                reasoning_mode,
            );
            super::openai_responses::collect_silent(
                &config,
                cancel,
                max_text_bytes,
                measurement.as_mut(),
            )
            .await
        }
        super::stream_dispatch::ClientKind::ChatCompletions => {
            let config = request_config(
                connection_id,
                fast_mode,
                model,
                messages,
                max_tokens,
                purpose,
                session_id,
                reasoning_mode,
            );
            match post_chat_request_with_timeout_measured(
                &config,
                request_timeout,
                measurement.as_mut(),
                Some(request_id),
            )
            .await
            {
                Ok(response) => {
                    super::stream_consume::consume_silent_bounded(
                        response,
                        cancel,
                        idle_timeout,
                        transport.usage_context(model),
                        transport.fragment_mode,
                        transport.error_policy,
                        max_text_bytes,
                        measurement.as_mut(),
                    )
                    .await
                }
                Err(error) => Err(error.to_string()),
            }
        }
        super::stream_dispatch::ClientKind::OllamaLocal => {
            let request = crate::services::agent_local::ollama_collect::collect_chat_with_timeout_and_limit_global(
                model,
                messages.to_vec(),
                request_timeout,
                Some(max_tokens),
            );
            tokio::select! {
                _ = cancel.cancelled() => Err("Annulé".to_string()),
                result = request => result.map(|(content, eval_count)| StreamResult {
                    content,
                    eval_count: Some(eval_count),
                    done_reason: Some("stop".to_string()),
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
    connection_id: &'a str,
    fast_mode: super::fast_mode::FastModeRequest,
    model: &'a str,
    messages: &'a [ChatMessage],
    max_tokens: u32,
    purpose: super::request_purpose::RequestPurpose,
    session_id: &'a str,
    reasoning_mode: Option<&'a str>,
) -> RequestConfig<'a> {
    RequestConfig {
        provider_id: connection_id,
        fast_mode,
        model,
        messages,
        tools: &[],
        think: reasoning_mode.is_some(),
        reasoning_mode,
        max_tokens: Some(max_tokens),
        purpose,
        session_id: Some(session_id),
        tool_result_previews: None,
        continuation_target: None,
    }
}
