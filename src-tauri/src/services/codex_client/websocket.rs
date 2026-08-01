use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::LazyLock;
use std::time::{Duration, Instant};

use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message as WsMessage;
use tokio_util::sync::CancellationToken;

use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::{ChatMessage, StreamOutcome};
use crate::services::compress::realtime_budget::RealtimeBudget;
use crate::services::secure_http::LLM_BODY_LIMIT;

use super::stream::CODEX_IDLE_TIMEOUT_SECS;
use super::stream_accumulator::StreamAccumulator;
use super::types::CodexRequest;
use super::{request, websocket_connect};

const SEND_TIMEOUT: Duration = Duration::from_secs(5);
const WEBSOCKET_COOLDOWN_MS: u64 = 5 * 60 * 1_000;
static PROCESS_START: LazyLock<Instant> = LazyLock::new(Instant::now);
static DISABLED_UNTIL_MS: AtomicU64 = AtomicU64::new(0);

#[allow(clippy::too_many_arguments)]
pub(super) async fn stream_chat(
    on_event: &AgentEventEmitter,
    session_id: &str,
    model: &str,
    messages: &[ChatMessage],
    tools: &[serde_json::Value],
    reasoning_mode: Option<&str>,
    cancel: CancellationToken,
    buffer_content: bool,
    realtime_budget: Option<RealtimeBudget>,
) -> Result<StreamOutcome, WebSocketFailure> {
    let request = request::build_codex_request(model, messages, tools, reasoning_mode);
    let payload = build_payload(&request)?;
    let mut socket = websocket_connect::connect(session_id)
        .await
        .map_err(|_| WebSocketFailure::Unavailable { partial: false })?;
    send_payload(&mut socket, payload, &cancel).await?;
    receive_response(
        &mut socket,
        on_event,
        tools,
        cancel,
        buffer_content,
        realtime_budget,
    )
    .await
}

pub(super) fn should_attempt() -> bool {
    !cooldown_active(elapsed_ms(), DISABLED_UNTIL_MS.load(Ordering::Relaxed))
}

pub(super) fn mark_unavailable() {
    DISABLED_UNTIL_MS.store(cooldown_deadline(elapsed_ms()), Ordering::Relaxed);
}

pub(super) fn mark_available() {
    DISABLED_UNTIL_MS.store(0, Ordering::Relaxed);
}

fn build_payload(request: &CodexRequest) -> Result<String, WebSocketFailure> {
    let mut payload = serde_json::to_value(request)
        .map_err(|_| WebSocketFailure::Unavailable { partial: false })?;
    let object = payload
        .as_object_mut()
        .ok_or(WebSocketFailure::Unavailable { partial: false })?;
    object.insert("type".to_string(), "response.create".into());
    let payload = serde_json::to_string(&payload)
        .map_err(|_| WebSocketFailure::Unavailable { partial: false })?;
    if payload.len() > LLM_BODY_LIMIT {
        return Err(WebSocketFailure::Unavailable { partial: false });
    }
    Ok(payload)
}

async fn send_payload(
    socket: &mut websocket_connect::CodexSocket,
    payload: String,
    cancel: &CancellationToken,
) -> Result<(), WebSocketFailure> {
    tokio::select! {
        _ = cancel.cancelled() => Err(WebSocketFailure::Cancelled),
        sent = tokio::time::timeout(SEND_TIMEOUT, socket.send(WsMessage::Text(payload.into()))) => {
            sent.map_err(|_| WebSocketFailure::Unavailable { partial: false })?
                .map_err(|_| WebSocketFailure::Unavailable { partial: false })
        }
    }
}

async fn receive_response(
    socket: &mut websocket_connect::CodexSocket,
    on_event: &AgentEventEmitter,
    tools: &[serde_json::Value],
    cancel: CancellationToken,
    buffer_content: bool,
    realtime_budget: Option<RealtimeBudget>,
) -> Result<StreamOutcome, WebSocketFailure> {
    let idle = Duration::from_secs(CODEX_IDLE_TIMEOUT_SECS);
    let mut deadline = tokio::time::Instant::now() + idle;
    let mut accumulator = StreamAccumulator::new(tools, buffer_content, realtime_budget);
    let mut partial = false;
    loop {
        let message = tokio::select! {
            _ = cancel.cancelled() => return Err(WebSocketFailure::Cancelled),
            _ = tokio::time::sleep_until(deadline) => {
                return Err(WebSocketFailure::Unavailable { partial });
            }
            message = socket.next() => message,
        };
        match message {
            Some(Ok(WsMessage::Text(text))) => {
                partial = true;
                if text.trim() == "[DONE]" {
                    return Err(WebSocketFailure::Unavailable { partial });
                }
                let parsed = serde_json::from_str(&text)
                    .map_err(|_| WebSocketFailure::Unavailable { partial })?;
                let outcome = accumulator
                    .apply(on_event, &parsed)
                    .map_err(|_| WebSocketFailure::Unavailable { partial })?;
                deadline = tokio::time::Instant::now() + idle;
                if let Some(outcome) = outcome {
                    return Ok(outcome);
                }
            }
            Some(Ok(WsMessage::Ping(payload))) => {
                send_pong(socket, payload, &cancel, partial).await?;
            }
            Some(Ok(WsMessage::Pong(_) | WsMessage::Frame(_))) => {}
            Some(Ok(WsMessage::Binary(_) | WsMessage::Close(_))) | Some(Err(_)) | None => {
                return Err(WebSocketFailure::Unavailable { partial });
            }
        }
    }
}

async fn send_pong(
    socket: &mut websocket_connect::CodexSocket,
    payload: tokio_tungstenite::tungstenite::Bytes,
    cancel: &CancellationToken,
    partial: bool,
) -> Result<(), WebSocketFailure> {
    tokio::select! {
        _ = cancel.cancelled() => Err(WebSocketFailure::Cancelled),
        sent = tokio::time::timeout(SEND_TIMEOUT, socket.send(WsMessage::Pong(payload))) => {
            sent.map_err(|_| WebSocketFailure::Unavailable { partial })?
                .map_err(|_| WebSocketFailure::Unavailable { partial })
        }
    }
}

fn elapsed_ms() -> u64 {
    u64::try_from(PROCESS_START.elapsed().as_millis()).unwrap_or(u64::MAX)
}

fn cooldown_deadline(now_ms: u64) -> u64 {
    now_ms.saturating_add(WEBSOCKET_COOLDOWN_MS)
}

fn cooldown_active(now_ms: u64, disabled_until_ms: u64) -> bool {
    disabled_until_ms != 0 && now_ms < disabled_until_ms
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum WebSocketFailure {
    Cancelled,
    Unavailable { partial: bool },
}

impl WebSocketFailure {
    pub(super) fn has_partial_output(self) -> bool {
        matches!(self, Self::Unavailable { partial: true })
    }
}

#[cfg(test)]
#[path = "websocket_tests.rs"]
mod tests;
