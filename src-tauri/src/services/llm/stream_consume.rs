#![expect(
    clippy::too_many_arguments,
    reason = "orchestration boundary keeps related runtime context explicit"
)]
use super::{
    stream_chunk::{self, ParsedChunk},
    stream_sse::is_done_marker,
    stream_tools::ToolCallAccumulator,
};
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::{StreamEvent, StreamOutcome, StreamResult};
use crate::services::stream_utils::{FilteredChunk, ThinkTagFilter};
use eventsource_stream::Eventsource;
use futures_util::StreamExt;
use tokio_util::sync::CancellationToken;

pub(super) async fn consume_stream(
    on_event: &AgentEventEmitter,
    resp: reqwest::Response,
    cancel: CancellationToken,
    buffer_content: bool,
    mut realtime_budget: Option<crate::services::compress::realtime_budget::RealtimeBudget>,
    tools: &[serde_json::Value],
    usage_context: crate::services::provider_usage::UsageContext<'_>,
    mut reasoning_capture: Option<super::reasoning_wire::ReasoningCapture>,
    mut measurement: Option<&mut crate::services::provider_usage::RequestMeasurement>,
) -> Result<StreamOutcome, String> {
    let mut stream = resp.bytes_stream().eventsource();
    let mut result = StreamResult::default();
    let mut token_count = 0;
    let mut acc = ToolCallAccumulator::new();
    let mut think_filter = ThinkTagFilter::new();
    let mut interrupted = false;

    loop {
        tokio::select! {
            _ = cancel.cancelled() => return Err("Annulé".to_string()),
            _ = tokio::time::sleep(super::timeouts::idle_timeout_for(
                usage_context.canonical_provider_id,
            )) => {
                return Err("provider_temporarily_unavailable".to_string());
            }
            event = stream.next() => {
                let Some(event) = event else {
                    return Err("provider_connection_failed".to_string());
                };
                let event = event.map_err(|_| "provider_connection_failed".to_string())?;
                if is_done_marker(&event.data) { break; }
                let value = super::stream_sse::parse_json(&event.data)?;
                if let Some(measurement) = measurement.as_mut() {
                    measurement.mark_first_event();
                    measurement.observe_response_metadata(&value);
                }
                if let Some(capture) = reasoning_capture.as_mut() {
                    capture.observe_json(&value);
                    capture.observe_done(&value);
                }
                let useful = process_chunk(
                    &value, on_event, &mut token_count, &mut result,
                    &mut acc, &mut think_filter, buffer_content, usage_context,
                )?;
                if useful {
                    if let Some(measurement) = measurement.as_mut() {
                        measurement.mark_first_useful();
                    }
                }
                if super::stream_consume_budget::should_interrupt(
                    &mut realtime_budget,
                    token_count,
                    acc.has_pending(),
                ) {
                    interrupted = true;
                    break;
                }
            }
        }
    }

    for chunk in think_filter.flush() {
        record_filtered(
            chunk,
            on_event,
            &mut result,
            &mut token_count,
            buffer_content,
        );
    }

    let (tool_calls, ids, extra_content) = acc.finalize();
    for (index, (wire_name, arguments)) in tool_calls.iter().enumerate() {
        let name = super::tool_schema::restore_tool_name(wire_name, tools);
        crate::services::agent_local::stream_buffer::record_tool_call_generation(
            on_event,
            &mut result,
            &name,
            arguments,
            &mut token_count,
        );
        let _ = on_event.send(StreamEvent::ToolCall {
            name: name.clone(),
            arguments: arguments.clone(),
            tool_call_index: index,
            tool_call_id: ids.get(index).cloned(),
            domain: crate::services::agent_local::memory_tool::event_domain(&name, arguments),
        });
        result.tool_calls.push((name, arguments.clone()));
        if let Some(id) = ids.get(index) {
            result.tool_call_ids.push(id.clone());
        }
        result
            .tool_call_extra_content
            .push(extra_content.get(index).cloned().flatten());
    }
    result.continuation = if interrupted {
        reasoning_capture.and_then(|mut capture| capture.finish_partial())
    } else {
        reasoning_capture.and_then(|mut capture| capture.finish_complete())
    };

    Ok(if interrupted {
        StreamOutcome::InterruptedForCompression(result)
    } else {
        StreamOutcome::Completed(result)
    })
}

fn process_chunk(
    value: &serde_json::Value,
    on_event: &AgentEventEmitter,
    token_count: &mut u32,
    result: &mut StreamResult,
    acc: &mut ToolCallAccumulator,
    think_filter: &mut ThinkTagFilter,
    buffer_content: bool,
    usage_context: crate::services::provider_usage::UsageContext<'_>,
) -> Result<bool, String> {
    let mut useful = false;
    for chunk in stream_chunk::parse_value_with_context(value, usage_context) {
        match chunk {
            ParsedChunk::Thinking(thinking) => {
                useful = true;
                crate::services::agent_local::stream_buffer::record_thinking(
                    on_event,
                    result,
                    thinking,
                    token_count,
                );
            }
            ParsedChunk::Content(content) => {
                useful = true;
                crate::services::agent_local::stream_buffer::record_generation_started(
                    on_event, result,
                );
                for filtered in think_filter.feed(&content) {
                    record_filtered(filtered, on_event, result, token_count, buffer_content);
                }
            }
            ParsedChunk::ToolCalls(tool_calls) => {
                if !tool_calls.is_empty() {
                    useful = true;
                    crate::services::agent_local::stream_buffer::record_generation_started(
                        on_event, result,
                    );
                }
                acc.ingest(&tool_calls);
            }
            ParsedChunk::Usage(usage) => {
                result.eval_count = usage.output_tokens.and_then(|value| value.try_into().ok());
                result.prompt_tokens = usage.input_tokens.and_then(|value| value.try_into().ok());
                result.usage = Some(usage);
            }
            ParsedChunk::GenerationDuration(duration_ns) => {
                result.generation.record_native_duration(duration_ns);
            }
            ParsedChunk::ProviderError(status) => {
                return Err(stream_chunk::provider_error_code(status).to_string());
            }
        }
    }
    Ok(useful)
}

fn record_filtered(
    chunk: FilteredChunk,
    on_event: &AgentEventEmitter,
    result: &mut StreamResult,
    token_count: &mut u32,
    buffer_content: bool,
) {
    match chunk {
        FilteredChunk::Thinking(content) => {
            crate::services::agent_local::stream_buffer::record_thinking(
                on_event,
                result,
                content,
                token_count,
            );
        }
        FilteredChunk::Content(content) => {
            crate::services::agent_local::stream_buffer::record_content(
                on_event,
                result,
                content,
                token_count,
                buffer_content,
            );
        }
    }
}

#[cfg(test)]
#[path = "stream_consume_tests.rs"]
mod tests;
