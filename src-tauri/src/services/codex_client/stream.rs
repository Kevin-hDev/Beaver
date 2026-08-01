use crate::services::agent_local::stream_buffer::StreamEventSink;
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::{ChatMessage, StreamOutcome};
use crate::services::compress::realtime_budget::RealtimeBudget;
use eventsource_stream::Eventsource;
use futures_util::StreamExt;
use tokio_util::sync::CancellationToken;

use super::limits::STREAM_STALL_TIMEOUT;
use super::{request, stream_accumulator::StreamAccumulator, stream_protocol, websocket};

pub use super::stream_silent::{collect_chat_silent, collect_chat_silent_for_compression};

pub async fn stream_chat_with_budget(
    on_event: &AgentEventEmitter,
    session_id: &str,
    request_id: &str,
    model: &str,
    messages: &[ChatMessage],
    tools: &[serde_json::Value],
    reasoning_mode: Option<&str>,
    cancel: CancellationToken,
    buffer_content: bool,
    realtime_budget: Option<RealtimeBudget>,
) -> Result<StreamOutcome, String> {
    if websocket::should_attempt() {
        match websocket::stream_chat(
            on_event,
            session_id,
            model,
            messages,
            tools,
            reasoning_mode,
            cancel.clone(),
            buffer_content,
            realtime_budget.clone(),
        )
        .await
        {
            Ok(outcome) => {
                websocket::mark_available();
                return Ok(outcome);
            }
            Err(websocket::WebSocketFailure::Cancelled) => return Err("Annulé".to_string()),
            Err(error) => {
                websocket::mark_unavailable();
                crate::services::agent_local::stream_diagnostics::record_retry(
                    session_id,
                    request_id,
                    "Repli HTTPS après indisponibilité du transport WebSocket.",
                )
                .await;
                if error.has_partial_output() {
                    crate::services::agent_local::ollama_retry_indicator::send_retry_indicator(
                        on_event,
                        crate::services::agent_local::ollama_retry_indicator::REASON_PROVIDER,
                        1,
                        1,
                    );
                }
            }
        }
    }
    let resp = request::post_codex_stream(model, messages, tools, reasoning_mode, &cancel).await?;
    consume_sse(
        on_event,
        resp,
        cancel,
        buffer_content,
        realtime_budget,
        tools,
    )
    .await
}

async fn consume_sse(
    on_event: &impl StreamEventSink,
    resp: reqwest::Response,
    cancel: CancellationToken,
    buffer_content: bool,
    realtime_budget: Option<RealtimeBudget>,
    tools: &[serde_json::Value],
) -> Result<StreamOutcome, String> {
    consume_sse_with_timeout(
        on_event,
        resp,
        cancel,
        buffer_content,
        realtime_budget,
        tools,
        STREAM_STALL_TIMEOUT,
    )
    .await
}

async fn consume_sse_with_timeout(
    on_event: &impl StreamEventSink,
    resp: reqwest::Response,
    cancel: CancellationToken,
    buffer_content: bool,
    realtime_budget: Option<RealtimeBudget>,
    tools: &[serde_json::Value],
    idle_timeout: std::time::Duration,
) -> Result<StreamOutcome, String> {
    let mut sse = resp.bytes_stream().eventsource();
    let mut accumulator = StreamAccumulator::new(tools, buffer_content, realtime_budget);
    loop {
        let event = tokio::select! {
            _ = cancel.cancelled() => return Err("Annulé".to_string()),
            _ = tokio::time::sleep(idle_timeout) => {
                return Err("provider_temporarily_unavailable".to_string());
            }
            ev = sse.next() => match ev {
                Some(Ok(e)) => e,
                Some(Err(_)) => return Err("provider_connection_failed".to_string()),
                None => return Err(stream_protocol::closed_before_completed()),
            },
        };

        if event.data.trim() == "[DONE]" {
            break;
        }
        let parsed: serde_json::Value = serde_json::from_str(&event.data)
            .map_err(|_| "provider_connection_failed".to_string())?;
        if let Some(outcome) = accumulator.apply(on_event, &parsed)? {
            return Ok(outcome);
        }
    }
    Err(stream_protocol::closed_before_completed())
}

#[cfg(test)]
#[path = "stream_tests.rs"]
mod tests;
