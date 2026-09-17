#![expect(clippy::too_many_arguments, reason = "explicit stream context")]
use super::{
    stream_chat_accumulator::{ChatStreamAccumulator, OutputMode},
    stream_sse::is_done_marker,
};
use crate::services::agent_local::stream_buffer::{DiscardStreamEvents, StreamEventSink};
use crate::services::agent_local::types_ollama::{StreamOutcome, StreamResult};
use eventsource_stream::Eventsource;
use futures_util::StreamExt;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

pub(super) async fn consume_stream(
    on_event: &impl StreamEventSink,
    resp: reqwest::Response,
    cancel: CancellationToken,
    buffer_content: bool,
    realtime_budget: Option<crate::services::compress::realtime_budget::RealtimeBudget>,
    tools: &[serde_json::Value],
    usage_context: crate::services::provider_usage::UsageContext<'_>,
    fragment_mode: super::route_profile::FragmentMode,
    error_policy: super::route_profile::ErrorPolicy,
    reasoning_capture: Option<super::reasoning_wire::ReasoningCapture>,
    measurement: Option<&mut crate::services::provider_usage::RequestMeasurement>,
) -> Result<StreamOutcome, String> {
    consume(
        on_event,
        resp,
        cancel,
        realtime_budget,
        tools,
        usage_context,
        fragment_mode,
        error_policy,
        reasoning_capture,
        measurement,
        super::timeouts::idle_timeout_for(usage_context.canonical_provider_id),
        OutputMode::Interactive { buffer_content },
    )
    .await
}

#[cfg(test)]
pub(super) async fn consume_silent(
    resp: reqwest::Response,
    cancel: CancellationToken,
    idle_timeout: Duration,
    usage_context: crate::services::provider_usage::UsageContext<'_>,
    fragment_mode: super::route_profile::FragmentMode,
    error_policy: super::route_profile::ErrorPolicy,
    measurement: Option<&mut crate::services::provider_usage::RequestMeasurement>,
) -> Result<StreamResult, String> {
    consume_silent_bounded(
        resp,
        cancel,
        idle_timeout,
        usage_context,
        fragment_mode,
        error_policy,
        usize::MAX,
        measurement,
    )
    .await
}

pub(super) async fn consume_silent_bounded(
    resp: reqwest::Response,
    cancel: CancellationToken,
    idle_timeout: Duration,
    usage_context: crate::services::provider_usage::UsageContext<'_>,
    fragment_mode: super::route_profile::FragmentMode,
    error_policy: super::route_profile::ErrorPolicy,
    max_text_bytes: usize,
    measurement: Option<&mut crate::services::provider_usage::RequestMeasurement>,
) -> Result<StreamResult, String> {
    consume(
        &DiscardStreamEvents,
        resp,
        cancel,
        None,
        &[],
        usage_context,
        fragment_mode,
        error_policy,
        None,
        measurement,
        idle_timeout,
        OutputMode::Silent { max_text_bytes },
    )
    .await
    .map(StreamOutcome::into_result)
}

async fn consume(
    on_event: &impl StreamEventSink,
    mut resp: reqwest::Response,
    cancel: CancellationToken,
    mut realtime_budget: Option<crate::services::compress::realtime_budget::RealtimeBudget>,
    tools: &[serde_json::Value],
    usage_context: crate::services::provider_usage::UsageContext<'_>,
    fragment_mode: super::route_profile::FragmentMode,
    error_policy: super::route_profile::ErrorPolicy,
    mut reasoning_capture: Option<super::reasoning_wire::ReasoningCapture>,
    mut measurement: Option<&mut crate::services::provider_usage::RequestMeasurement>,
    idle_timeout: Duration,
    output_mode: OutputMode,
) -> Result<StreamOutcome, String> {
    let mut routing = super::provider_diagnostics::openrouter::take(&mut resp);
    let stream = super::stream_sse::bounded_response(resp).eventsource();
    futures_util::pin_mut!(stream);
    let mut accumulator = ChatStreamAccumulator::new(fragment_mode, output_mode);
    let mut interrupted = false;

    loop {
        tokio::select! {
            _ = cancel.cancelled() => return Err("Annulé".to_string()),
            _ = tokio::time::sleep(idle_timeout) => {
                return Err("provider_temporarily_unavailable".to_string());
            }
            event = stream.next() => {
                let Some(event) = event else {
                    return Err("provider_connection_failed".to_string());
                };
                let event = event.map_err(|_| "provider_connection_failed".to_string())?;
                if is_done_marker(&event.data) {
                    if let Some(capture) = reasoning_capture.as_mut() {
                        capture.observe_transport_complete();
                    }
                    break;
                }
                let value = super::stream_sse::parse_json(&event.data)?;
                if let Some(routing) = routing.as_mut() {
                    routing.observe(&value);
                }
                if let Some(measurement) = measurement.as_mut() {
                    measurement.mark_first_event();
                    measurement.observe_response_metadata(&value);
                }
                if let Some(capture) = reasoning_capture.as_mut() {
                    capture.observe_json(&value);
                    capture.observe_done(&value);
                }
                let useful = accumulator.apply(on_event, &value, usage_context, error_policy)?;
                if useful {
                    if let Some(measurement) = measurement.as_mut() {
                        measurement.mark_first_useful();
                    }
                }
                if super::stream_consume_budget::should_interrupt(
                    &mut realtime_budget,
                    accumulator.output_tokens(),
                    accumulator.has_pending_tools(),
                ) {
                    interrupted = true;
                    break;
                }
            }
        }
    }

    let mut result = accumulator.finish(
        on_event,
        usage_context.canonical_provider_id,
        tools,
        interrupted,
    )?;
    result.continuation = reasoning_capture.and_then(|mut capture| {
        if interrupted || result.completion_error.is_some() {
            capture.finish_partial()
        } else {
            capture.observe_persisted_tool_links(&result.tool_calls, &result.tool_call_ids);
            capture.finish_complete()
        }
    });
    Ok(if interrupted {
        StreamOutcome::InterruptedForCompression(result)
    } else {
        StreamOutcome::Completed(result)
    })
}

#[cfg(test)]
#[path = "stream_consume_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "stream_silent_consume_tests.rs"]
mod silent_tests;
