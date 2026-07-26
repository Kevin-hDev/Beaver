use super::ollama_base_url;
use super::ollama_retry_indicator::{
    send_retry_indicator, server_retry_delay, should_retry_server_status, MAX_SERVER_RETRIES,
    REASON_FEATURE_DROPPED, REASON_PARSER_CRASH, REASON_SERVER,
};
use super::ollama_stream_retry::build_retry_request;
use super::ollama_tool_parse_retry::{is_tool_parse_crash, MAX_PARSER_RETRIES};
use super::ollama_tool_role::wrap_tool_results;
use super::ollama_wire;
use super::stream_events::AgentEventEmitter;
use super::types_ollama::{ChatRequest, StreamEvent};
use crate::services::llm::vision;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone, Copy)]
pub struct RetryCounts {
    pub parser_retries: u32,
    pub server_retries: u32,
}

pub enum OpenChatResponse {
    Ready(reqwest::Response),
    Retry {
        request: ChatRequest,
        counts: RetryCounts,
    },
}

pub async fn open_chat_response(
    on_event: &AgentEventEmitter,
    request: &ChatRequest,
    cancel: &CancellationToken,
    counts: RetryCounts,
    emit_retry_indicator: bool,
) -> Result<OpenChatResponse, String> {
    let wire_messages = wrap_tool_results(&request.messages);
    let wire_request = ollama_wire::chat_request(request, &wire_messages);

    let client = reqwest::Client::new();
    let resp = match client
        .post(format!("{}/api/chat", ollama_base_url()))
        .json(&wire_request)
        .send()
        .await
    {
        Ok(response) => response,
        Err(error) => return connection_error(on_event, error),
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
        eprintln!("[ollama-stream] modèle sans {feature}, retry");
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
        eprintln!("[ollama-stream] crash parser tool-call (#{attempt}), retry");
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

    if should_retry_server_status(status) && counts.server_retries < MAX_SERVER_RETRIES {
        let attempt = counts.server_retries + 1;
        eprintln!("[ollama-stream] HTTP {status}, retry serveur #{attempt}");
        maybe_send_retry_indicator(
            on_event,
            emit_retry_indicator,
            REASON_SERVER,
            attempt,
            MAX_SERVER_RETRIES,
        );
        wait_retry_delay(cancel, attempt).await?;
        return Ok(OpenChatResponse::Retry {
            request: request.clone(),
            counts: RetryCounts {
                server_retries: attempt,
                ..counts
            },
        });
    }

    eprintln!("[ollama-stream] HTTP {status}: {body}");
    let msg = "ollama_server_error".to_string();
    let _ = on_event.send(StreamEvent::Error {
        message: msg.clone(),
        is_connection: false,
        diagnostic: None,
    });
    Err(msg)
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

fn connection_error(
    on_event: &AgentEventEmitter,
    error: reqwest::Error,
) -> Result<OpenChatResponse, String> {
    let is_connection = error.is_connect() || error.is_timeout();
    let msg = if is_connection {
        "ollama_connection_lost".to_string()
    } else {
        format!("Ollama: {error}")
    };
    let _ = on_event.send(StreamEvent::Error {
        message: msg.clone(),
        is_connection,
        diagnostic: None,
    });
    Err(msg)
}

async fn wait_retry_delay(cancel: &CancellationToken, attempt: u32) -> Result<(), String> {
    tokio::select! {
        _ = cancel.cancelled() => Err("Annulé".to_string()),
        _ = tokio::time::sleep(server_retry_delay(attempt)) => Ok(()),
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
