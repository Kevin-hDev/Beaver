#![expect(
    clippy::too_many_arguments,
    reason = "orchestration boundary keeps related runtime context explicit"
)]
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message as WsMessage;
use tokio_util::sync::CancellationToken;

use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::{ChatMessage, StreamOutcome};
use crate::services::compress::realtime_budget::RealtimeBudget;

use super::limits::STREAM_STALL_TIMEOUT;
use super::stream_accumulator::StreamAccumulator;
use super::stream_measurement::StreamMeasurement;
use super::types::CodexRequest;
use super::{request, websocket_connect};

#[path = "websocket_payload.rs"]
mod payload;
use payload::build_payload;

const SEND_TIMEOUT: Duration = Duration::from_secs(5);
pub(super) use super::websocket_state::WebSocketFailure;
pub(super) use super::websocket_state::{
    accumulator_failure, mark_available, mark_unavailable, should_attempt,
};
#[cfg(test)]
use super::websocket_state::{cooldown_active, cooldown_deadline, WEBSOCKET_COOLDOWN_MS};

#[expect(
    clippy::too_many_arguments,
    reason = "boundary parameters remain explicit and locally audited"
)]
pub(super) async fn stream_chat(
    on_event: &AgentEventEmitter,
    session_id: &str,
    model: &str,
    messages: &[ChatMessage],
    tools: &[serde_json::Value],
    reasoning_mode: Option<&str>,
    fast_mode: crate::services::llm::fast_mode::FastModeRequest,
    cancel: CancellationToken,
    buffer_content: bool,
    realtime_budget: Option<RealtimeBudget>,
    measurement: &mut StreamMeasurement<'_>,
    preparation: Option<
        &crate::services::agent_local::context_usage_runtime::PreparedContextAttempt<'_>,
    >,
) -> Result<StreamOutcome, WebSocketFailure> {
    let mut request = request::build_codex_request(
        model,
        messages,
        tools,
        reasoning_mode,
        Some(session_id),
        fast_mode,
    );
    tokio::select! {
        _ = cancel.cancelled() => return Err(WebSocketFailure::Cancelled),
        result = super::model_catalog::reasoning::prepare(&mut request) => {
            result.map_err(|_| configuration_rejected())?;
        }
    }
    if let Some(preparation) = preparation {
        let payload = serde_json::to_value(&request).map_err(|_| configuration_rejected())?;
        preparation
            .persist_payload(
                crate::services::agent_local::prepared_context_count::responses(&payload),
            )
            .await
            .map_err(|_| configuration_rejected())?;
    }
    let payload = build_payload(&request)?;
    let routing_hint =
        super::routing_hint::for_request(&request).map_err(|_| configuration_rejected())?;
    // Use the same derived affinity as the HTTP path and payload, never the local UUID.
    let cache_key = request
        .prompt_cache_key
        .as_deref()
        .ok_or_else(configuration_rejected)?;
    let mut socket = websocket_connect::connect(cache_key, &routing_hint)
        .await
        .map_err(|_| WebSocketFailure::Unavailable { partial: false })?;
    measurement.mark_headers();
    send_payload(&mut socket, payload, &cancel).await?;
    receive_response(
        &mut socket,
        on_event,
        model,
        tools,
        cancel,
        buffer_content,
        realtime_budget,
        measurement,
    )
    .await
}

fn configuration_rejected() -> WebSocketFailure {
    WebSocketFailure::ProviderRejected {
        code: crate::services::llm::provider_error::ProviderErrorCode::ProviderConfigurationInvalid,
    }
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
    model: &str,
    tools: &[serde_json::Value],
    cancel: CancellationToken,
    buffer_content: bool,
    realtime_budget: Option<RealtimeBudget>,
    measurement: &mut StreamMeasurement<'_>,
) -> Result<StreamOutcome, WebSocketFailure> {
    let idle = STREAM_STALL_TIMEOUT;
    let mut deadline = tokio::time::Instant::now() + idle;
    let mut accumulator =
        StreamAccumulator::new("openai", model, tools, buffer_content, realtime_budget);
    let mut partial = false;
    loop {
        let message = tokio::select! {
            biased;
            _ = cancel.cancelled() => return Err(WebSocketFailure::Cancelled),
            _ = tokio::time::sleep_until(deadline) => {
                return Err(WebSocketFailure::Unavailable { partial });
            }
            message = socket.next() => message,
        };
        match message {
            Some(Ok(WsMessage::Text(text))) => {
                if text.trim() == "[DONE]" {
                    return Err(WebSocketFailure::Unavailable { partial });
                }
                let parsed = serde_json::from_str(&text)
                    .map_err(|_| WebSocketFailure::Unavailable { partial })?;
                let applied = measurement.apply(&mut accumulator, on_event, &parsed);
                partial = accumulator.has_partial_output();
                let outcome = applied.map_err(|error| accumulator_failure(&error, partial))?;
                deadline = tokio::time::Instant::now() + idle;
                if let Some(outcome) = outcome {
                    return Ok(outcome);
                }
            }
            Some(Ok(WsMessage::Ping(payload))) => {
                // Un ping confirme le transport, pas l'avancement du modèle :
                // il ne réarme donc pas le délai d'inactivité sémantique.
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

#[cfg(test)]
#[path = "websocket_tests.rs"]
mod tests;
