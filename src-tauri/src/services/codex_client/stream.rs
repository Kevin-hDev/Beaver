#![expect(
    clippy::too_many_arguments,
    reason = "orchestration boundary keeps related runtime context explicit"
)]
use crate::services::agent_local::stream_buffer::StreamEventSink;
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::{ChatMessage, StreamOutcome};
use crate::services::compress::realtime_budget::RealtimeBudget;
use eventsource_stream::Eventsource;
use futures_util::StreamExt;
use tokio_util::sync::CancellationToken;

use super::limits::STREAM_STALL_TIMEOUT;
use super::{
    request, stream_accumulator::StreamAccumulator, stream_measurement::StreamMeasurement,
    stream_protocol, websocket,
};

pub use super::stream_silent::collect_chat_silent_for_compression;

pub async fn stream_chat_with_budget(
    on_event: &AgentEventEmitter,
    session_id: &str,
    request_id: &str,
    model: &str,
    messages: &[ChatMessage],
    tools: &[serde_json::Value],
    reasoning_mode: Option<&str>,
    fast_mode: crate::services::llm::fast_mode::FastModeRequest,
    cancel: CancellationToken,
    buffer_content: bool,
    realtime_budget: Option<RealtimeBudget>,
    measurement: Option<&mut crate::services::provider_usage::RequestMeasurement>,
) -> Result<StreamOutcome, String> {
    let mut measurement = StreamMeasurement::new(measurement);
    if websocket::should_attempt() {
        match websocket::stream_chat(
            on_event,
            session_id,
            model,
            messages,
            tools,
            reasoning_mode,
            fast_mode,
            cancel.clone(),
            buffer_content,
            realtime_budget.clone(),
            &mut measurement,
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
    let resp = request::post_codex_stream(
        model,
        messages,
        tools,
        reasoning_mode,
        Some(session_id),
        fast_mode,
        &cancel,
    )
    .await?;
    measurement.mark_headers();
    consume_sse(
        on_event,
        resp,
        cancel,
        buffer_content,
        realtime_budget,
        model,
        tools,
        &mut measurement,
    )
    .await
}

async fn consume_sse(
    on_event: &impl StreamEventSink,
    resp: reqwest::Response,
    cancel: CancellationToken,
    buffer_content: bool,
    realtime_budget: Option<RealtimeBudget>,
    model: &str,
    tools: &[serde_json::Value],
    measurement: &mut StreamMeasurement<'_>,
) -> Result<StreamOutcome, String> {
    consume_sse_with_timeout(
        on_event,
        resp,
        cancel,
        buffer_content,
        realtime_budget,
        "openai",
        model,
        tools,
        STREAM_STALL_TIMEOUT,
        measurement,
    )
    .await
}

pub(crate) async fn consume_external_responses_sse(
    on_event: &impl StreamEventSink,
    resp: reqwest::Response,
    cancel: CancellationToken,
    buffer_content: bool,
    realtime_budget: Option<RealtimeBudget>,
    provider: &str,
    model: &str,
    tools: &[serde_json::Value],
    measurement: Option<&mut crate::services::provider_usage::RequestMeasurement>,
) -> Result<StreamOutcome, String> {
    let mut measurement = StreamMeasurement::new(measurement);
    consume_sse_with_timeout(
        on_event,
        resp,
        cancel,
        buffer_content,
        realtime_budget,
        provider,
        model,
        tools,
        STREAM_STALL_TIMEOUT,
        &mut measurement,
    )
    .await
}

async fn consume_sse_with_timeout(
    on_event: &impl StreamEventSink,
    resp: reqwest::Response,
    cancel: CancellationToken,
    buffer_content: bool,
    realtime_budget: Option<RealtimeBudget>,
    provider: &str,
    model: &str,
    tools: &[serde_json::Value],
    idle_timeout: std::time::Duration,
    measurement: &mut StreamMeasurement<'_>,
) -> Result<StreamOutcome, String> {
    let mut sse = resp.bytes_stream().eventsource();
    let mut accumulator =
        StreamAccumulator::new(provider, model, tools, buffer_content, realtime_budget);
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
        let parsed = crate::services::llm::stream_sse::parse_json(&event.data)?;
        if let Some(outcome) = measurement.apply(&mut accumulator, on_event, &parsed)? {
            return Ok(outcome);
        }
    }
    Err(stream_protocol::closed_before_completed())
}

#[cfg(test)]
#[path = "stream_tests.rs"]
mod tests;
