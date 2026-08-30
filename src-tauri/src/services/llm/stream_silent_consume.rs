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

pub(super) async fn consume_silent(
    resp: reqwest::Response,
    cancel: CancellationToken,
    idle_timeout: Duration,
    usage_context: crate::services::provider_usage::UsageContext<'_>,
    fragment_mode: super::route_profile::FragmentMode,
    error_policy: super::route_profile::ErrorPolicy,
    mut measurement: Option<&mut crate::services::provider_usage::RequestMeasurement>,
) -> Result<StreamResult, String> {
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
                )?;
                if useful {
                    if let Some(measurement) = measurement.as_mut() {
                        measurement.mark_first_useful();
                    }
                }
            }
        }
    }
    flush_content(&mut result, &mut think_filter);
    finalize_tools(&mut result, acc);
    Ok(result)
}

fn process_chunk(
    data: &str,
    result: &mut StreamResult,
    acc: &mut ToolCallAccumulator,
    think_filter: &mut ThinkTagFilter,
    fragments: &mut super::stream_fragments::StreamFragmentState,
    usage_context: crate::services::provider_usage::UsageContext<'_>,
    error_policy: super::route_profile::ErrorPolicy,
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
                result.prompt_tokens = usage.input_tokens.and_then(|value| value.try_into().ok());
                result.usage = Some(usage);
            }
            ParsedChunk::GenerationDuration(_) => {}
            ParsedChunk::ProviderError(status) => {
                return Err(stream_chunk::provider_error_code(error_policy, status).to_string());
            }
        }
    }
    Ok(useful)
}

fn flush_content(result: &mut StreamResult, filter: &mut ThinkTagFilter) {
    for chunk in filter.flush() {
        if let FilteredChunk::Content(content) = chunk {
            result.content.push_str(&content);
        }
    }
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
