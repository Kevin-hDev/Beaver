use super::ollama_client::OllamaClient;
use super::ollama_retry_indicator::{
    max_server_retries, send_retry_indicator, should_retry_server_status, REASON_FEATURE_DROPPED,
    REASON_PARSER_CRASH, REASON_SERVER,
};
use super::ollama_stream_retry::build_retry_request;
use super::ollama_tool_parse_retry::{is_tool_parse_crash, MAX_PARSER_RETRIES};
use super::ollama_tool_role::wrap_tool_results;
use super::ollama_wire;
use super::stream_events::AgentEventEmitter;
use super::types_ollama::{ChatRequest, StreamEvent};
use crate::services::compress::realtime_budget::RealtimeBudget;
use crate::services::llm::vision;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

#[path = "ollama_stream_request_error.rs"]
pub(super) mod request_error;

#[derive(Debug, Clone, Copy)]
pub struct RetryCounts {
    pub parser_retries: u32,
    pub server_retries: u32,
}

pub(super) struct StreamChatOptions {
    pub(super) tool_tx: Option<mpsc::UnboundedSender<(usize, String, serde_json::Value)>>,
    pub(super) buffer_content: bool,
    pub(super) realtime_budget: Option<RealtimeBudget>,
    pub(super) retry_counts: RetryCounts,
}

#[derive(Clone, Copy)]
pub struct ReplayDiagnosticContext<'a> {
    pub session_id: &'a str,
    pub request_id: &'a str,
    pub preparation: Option<&'a super::context_usage_runtime::PreparedContextAttempt<'a>>,
}

pub enum OpenChatResponse {
    Ready(reqwest::Response),
    Retry {
        request: ChatRequest,
        counts: RetryCounts,
    },
}

pub async fn open_chat_response(
    ollama: &OllamaClient,
    on_event: &AgentEventEmitter,
    request: &ChatRequest,
    cancel: &CancellationToken,
    counts: RetryCounts,
    emit_retry_indicator: bool,
    diagnostics: ReplayDiagnosticContext<'_>,
) -> Result<OpenChatResponse, String> {
    let placement = crate::services::llm::route_profile::payload_policy("ollama", &request.model)
        .expect("Ollama route profile")
        .message
        .tool_results;
    let wire_messages = wrap_tool_results(&request.messages, placement);
    let prepared = ollama_wire::chat_request_with_evidence(request, &wire_messages)
        .map_err(|_| "reasoning_continuity_invalid".to_string())?;
    crate::services::llm::reasoning_wire::replay::record_evidence(
        Some(diagnostics.session_id),
        Some(diagnostics.request_id),
        &prepared.replayed,
    )
    .await;
    let context_count = prepared.context_count;
    let wire_request = prepared.payload;
    persist_verified_context(&context_count, diagnostics.preparation).await?;
    super::stream_diagnostics_payload::record_provider_payload(
        Some(diagnostics.session_id),
        Some(diagnostics.request_id),
        "ollama",
        "ollama_chat",
        &wire_request,
    )
    .await;
    #[cfg(debug_assertions)]
    crate::services::reasoning_fixture_budget::authorize_payload(&wire_request)?;

    let client = reqwest::Client::new();
    let base_url = ollama.base_url().await?;
    let resp = match client
        .post(format!("{base_url}/api/chat"))
        .json(&wire_request)
        .send()
        .await
    {
        Ok(response) => response,
        Err(error) => return request_error::connection(error),
    };

    if resp.status().is_success() {
        return Ok(OpenChatResponse::Ready(resp));
    }

    handle_http_failure(
        on_event,
        request,
        resp,
        cancel,
        counts,
        emit_retry_indicator,
    )
    .await
}

async fn persist_verified_context(
    count: &super::context_usage_record::ContextTokenCount,
    preparation: Option<&super::context_usage_runtime::PreparedContextAttempt<'_>>,
) -> Result<(), String> {
    if count.capacity_tokens.is_none() {
        return Err(super::context_capacity_error::UNVERIFIED_CODE.to_string());
    }
    if let Some(preparation) = preparation {
        preparation.persist_payload(count.clone()).await?;
    }
    Ok(())
}

async fn handle_http_failure(
    on_event: &AgentEventEmitter,
    request: &ChatRequest,
    resp: reqwest::Response,
    cancel: &CancellationToken,
    counts: RetryCounts,
    emit_retry_indicator: bool,
) -> Result<OpenChatResponse, String> {
    let status = resp.status();
    let body = resp.text().await.unwrap_or_default();

    if let Some(retry_req) = build_retry_request(request, &body) {
        let feature = feature_name(request, &retry_req);
        ::log::warn!("[ollama-stream] modèle sans {feature}, retry");
        maybe_send_retry_indicator(on_event, emit_retry_indicator, REASON_FEATURE_DROPPED, 1, 1);
        if feature == "images" {
            let _ = on_event.send(StreamEvent::Notice {
                message_key: vision::NOTICE_UNSUPPORTED_MODEL.to_string(),
            });
        }
        return Ok(OpenChatResponse::Retry {
            request: retry_req,
            counts,
        });
    }

    if is_tool_parse_crash(&body) && counts.parser_retries < MAX_PARSER_RETRIES {
        let attempt = counts.parser_retries + 1;
        ::log::warn!("[ollama-stream] crash parser tool-call (#{attempt}), retry");
        maybe_send_retry_indicator(
            on_event,
            emit_retry_indicator,
            REASON_PARSER_CRASH,
            attempt,
            MAX_PARSER_RETRIES,
        );
        return Ok(OpenChatResponse::Retry {
            request: request.clone(),
            counts: RetryCounts {
                parser_retries: attempt,
                ..counts
            },
        });
    }

    if should_retry_server_status(status, counts.server_retries) {
        let attempt = counts.server_retries + 1;
        ::log::warn!("[ollama-stream] HTTP {status}, retry serveur #{attempt}");
        maybe_send_retry_indicator(
            on_event,
            emit_retry_indicator,
            REASON_SERVER,
            attempt,
            max_server_retries(),
        );
        request_error::wait_retry(cancel, attempt).await?;
        return Ok(OpenChatResponse::Retry {
            request: request.clone(),
            counts: RetryCounts {
                server_retries: attempt,
                ..counts
            },
        });
    }

    ::log::error!(
        "[ollama-stream] HTTP {status}: {}",
        crate::services::llm::sanitize_log_body(&body)
    );
    Err(request_error::SERVER.to_string())
}

fn maybe_send_retry_indicator(
    on_event: &AgentEventEmitter,
    enabled: bool,
    reason_key: &str,
    attempt: u32,
    max_attempts: u32,
) {
    if enabled {
        send_retry_indicator(on_event, reason_key, attempt, max_attempts);
    }
}

fn feature_name(request: &ChatRequest, retry: &ChatRequest) -> &'static str {
    if retry.think != request.think {
        "thinking"
    } else if retry.tools != request.tools {
        "tools"
    } else {
        "images"
    }
}

#[cfg(test)]
#[path = "ollama_stream_request_tests.rs"]
mod tests;
