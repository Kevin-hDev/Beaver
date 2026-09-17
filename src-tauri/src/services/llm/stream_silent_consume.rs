use super::{
    stream_chunk::{self, ParsedChunk},
    stream_sse::is_done_marker,
    stream_tools::ToolCallAccumulator,
};
use crate::services::agent_local::types_ollama::StreamResult;
use crate::services::stream_utils::{FilteredChunk, ThinkTagFilter};
use eventsource_stream::Eventsource;
use futures_util::StreamExt;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

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

#[expect(
    clippy::too_many_arguments,
    reason = "the shared stream boundary keeps route policy, attribution and byte budget explicit"
)]
pub(super) async fn consume_silent_bounded(
    mut resp: reqwest::Response,
    cancel: CancellationToken,
    idle_timeout: Duration,
    usage_context: crate::services::provider_usage::UsageContext<'_>,
    fragment_mode: super::route_profile::FragmentMode,
    error_policy: super::route_profile::ErrorPolicy,
    max_text_bytes: usize,
    mut measurement: Option<&mut crate::services::provider_usage::RequestMeasurement>,
) -> Result<StreamResult, String> {
    let mut routing = super::provider_diagnostics::openrouter::take(&mut resp);
    if cancel.is_cancelled() {
        return Err("Annulé".to_string());
    }
    let stream = super::stream_sse::bounded_response(resp).eventsource();
    futures_util::pin_mut!(stream);
    let mut result = StreamResult::default();
    let mut acc = ToolCallAccumulator::new();
    let mut think_filter = ThinkTagFilter::new();
    let mut fragments = super::stream_fragments::StreamFragmentState::new(fragment_mode);

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
                if is_done_marker(&event.data) { break; }
                let value = super::stream_sse::parse_json(&event.data)?;
                if let Some(routing) = routing.as_mut() {
                    routing.observe(&value);
                }
                if let Some(measurement) = measurement.as_mut() {
                    measurement.mark_first_event();
                    measurement.observe_response_metadata(&value);
                }
                let useful = process_chunk(
                    &event.data,
                    &mut result,
                    &mut acc,
                    &mut think_filter,
                    &mut fragments,
                    usage_context,
                    error_policy,
                    max_text_bytes,
                )?;
                if useful {
                    if let Some(measurement) = measurement.as_mut() {
                        measurement.mark_first_useful();
                    }
                }
            }
        }
    }
    flush_content(&mut result, &mut think_filter, max_text_bytes)?;
    if super::stream_completion::terminal_error(&result).is_some() {
        acc = ToolCallAccumulator::new();
    }
    finalize_tools(&mut result, acc);
    super::stream_completion::finish(&mut result);
    Ok(result)
}

#[expect(
    clippy::too_many_arguments,
    reason = "chunk processing keeps its bounded stream state explicit"
)]
fn process_chunk(
    data: &str,
    result: &mut StreamResult,
    acc: &mut ToolCallAccumulator,
    think_filter: &mut ThinkTagFilter,
    fragments: &mut super::stream_fragments::StreamFragmentState,
    usage_context: crate::services::provider_usage::UsageContext<'_>,
    error_policy: super::route_profile::ErrorPolicy,
    max_text_bytes: usize,
) -> Result<bool, String> {
    let mut useful = false;
    for chunk in stream_chunk::parse_with_context(data, usage_context) {
        match chunk {
            ParsedChunk::Content(content) => {
                let content = fragments.content(&content)?;
                if content.is_empty() {
                    continue;
                }
                useful = true;
                for filtered in think_filter.feed(&content) {
                    if let FilteredChunk::Content(content) = filtered {
                        if result.content.len().saturating_add(content.len()) > max_text_bytes {
                            return Err("provider_payload_too_large".to_string());
                        }
                        result.content.push_str(&content);
                    }
                }
            }
            ParsedChunk::Thinking(_) => {}
            ParsedChunk::ToolCalls(tool_calls) => {
                useful |= !tool_calls.is_empty();
                acc.ingest(&tool_calls);
            }
            ParsedChunk::Usage(usage) => {
                result.eval_count = usage.output_tokens.and_then(|value| value.try_into().ok());
                result.prompt_tokens = usage.context_input_tokens(usage_context.api_format);
                result.usage = Some(usage);
            }
            ParsedChunk::GenerationDuration(_) => {}
            ParsedChunk::FinishReason(reason) => result.done_reason = Some(reason.into()),
            ParsedChunk::ProviderError(status) => {
                return Err(stream_chunk::provider_error_code(error_policy, status).to_string());
            }
        }
    }
    Ok(useful)
}

fn flush_content(
    result: &mut StreamResult,
    filter: &mut ThinkTagFilter,
    max_text_bytes: usize,
) -> Result<(), String> {
    for chunk in filter.flush() {
        if let FilteredChunk::Content(content) = chunk {
            if result.content.len().saturating_add(content.len()) > max_text_bytes {
                return Err("provider_payload_too_large".to_string());
            }
            result.content.push_str(&content);
        }
    }
    Ok(())
}

fn finalize_tools(result: &mut StreamResult, acc: ToolCallAccumulator) {
    let (tool_calls, ids, extra_content) = acc.finalize();
    for (index, (name, args)) in tool_calls.iter().enumerate() {
        result.tool_calls.push((name.clone(), args.clone()));
        if let Some(id) = ids.get(index) {
            result.tool_call_ids.push(id.clone());
        }
        result
            .tool_call_extra_content
            .push(extra_content.get(index).cloned().flatten());
    }
}

#[cfg(test)]
#[path = "stream_silent_consume_tests.rs"]
mod tests;
