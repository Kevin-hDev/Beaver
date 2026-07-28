use super::{
    stream_chunk::{self, ParsedChunk},
    stream_sse::is_done_marker,
    stream_tools::ToolCallAccumulator,
};
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::{StreamEvent, StreamOutcome, StreamResult};
use crate::services::compress::realtime_budget::RealtimeBudget;
use crate::services::stream_utils::FilteredChunk;
use crate::services::stream_utils::ThinkTagFilter;
use eventsource_stream::Eventsource;
use futures_util::StreamExt;
use tokio_util::sync::CancellationToken;

pub(super) async fn consume_stream(
    on_event: &AgentEventEmitter,
    resp: reqwest::Response,
    cancel: CancellationToken,
    buffer_content: bool,
    mut realtime_budget: Option<RealtimeBudget>,
    tools: &[serde_json::Value],
) -> Result<(StreamOutcome, u32, std::time::Instant), String> {
    let mut stream = resp.bytes_stream().eventsource();
    let mut result = StreamResult::default();
    let mut token_count: u32 = 0;
    let mut first_token: Option<std::time::Instant> = None;
    let mut acc = ToolCallAccumulator::new();
    let mut think_filter = ThinkTagFilter::new();
    let mut interrupted = false;

    loop {
        tokio::select! {
            _ = cancel.cancelled() => return Err("Annulé".to_string()),
            _ = tokio::time::sleep(super::timeouts::idle_timeout()) => {
                return Err("provider_temporarily_unavailable".to_string());
            }
            event = stream.next() => {
                let Some(event) = event else { break; };
                let event = event.map_err(|_| "provider_connection_failed".to_string())?;
                if is_done_marker(&event.data) { break; }
                process_chunk(&event.data, on_event, &mut token_count, &mut first_token, &mut result, &mut acc, &mut think_filter, buffer_content);
                if should_interrupt(&mut realtime_budget, token_count, acc.has_pending()) {
                    interrupted = true;
                    break;
                }
            }
        }
    }

    for chunk in think_filter.flush() {
        match chunk {
            FilteredChunk::Thinking(t) => {
                result.thinking.push_str(&t);
                let _ = on_event.send(StreamEvent::Thinking { content: t });
            }
            FilteredChunk::Content(c) => {
                crate::services::agent_local::stream_buffer::record_content(
                    on_event,
                    &mut result,
                    c,
                    &mut token_count,
                    &mut first_token,
                    buffer_content,
                );
            }
        }
    }

    let (tool_calls, ids, extra_content) = acc.finalize();
    for (i, (wire_name, args)) in tool_calls.iter().enumerate() {
        let name = super::tool_schema::restore_tool_name(wire_name, tools);
        let _ = on_event.send(StreamEvent::ToolCall {
            name: name.clone(),
            arguments: args.clone(),
            domain: crate::services::agent_local::memory_tool::event_domain(&name, args),
        });
        result.tool_calls.push((name, args.clone()));
        if let Some(id) = ids.get(i) {
            result.tool_call_ids.push(id.clone());
        }
        result
            .tool_call_extra_content
            .push(extra_content.get(i).cloned().flatten());
    }

    let first = first_token.unwrap_or_else(std::time::Instant::now);
    let outcome = if interrupted {
        StreamOutcome::InterruptedForCompression(result)
    } else {
        StreamOutcome::Completed(result)
    };
    Ok((outcome, token_count, first))
}

fn should_interrupt(
    budget: &mut Option<RealtimeBudget>,
    token_count: u32,
    has_pending_tool_call: bool,
) -> bool {
    !has_pending_tool_call
        && budget
            .as_mut()
            .is_some_and(|budget| budget.should_interrupt(token_count))
}

fn process_chunk(
    data: &str,
    on_event: &AgentEventEmitter,
    token_count: &mut u32,
    first_token: &mut Option<std::time::Instant>,
    result: &mut StreamResult,
    acc: &mut ToolCallAccumulator,
    think_filter: &mut ThinkTagFilter,
    buffer_content: bool,
) {
    for chunk in stream_chunk::parse(data) {
        match chunk {
            ParsedChunk::Thinking(thinking) => {
                result.thinking.push_str(&thinking);
                *token_count += 1;
                let _ = on_event.send(StreamEvent::Thinking { content: thinking });
            }
            ParsedChunk::Content(content) => {
                for filtered in think_filter.feed(&content) {
                    match filtered {
                        FilteredChunk::Thinking(t) => {
                            result.thinking.push_str(&t);
                            let _ = on_event.send(StreamEvent::Thinking { content: t });
                        }
                        FilteredChunk::Content(c) => {
                            crate::services::agent_local::stream_buffer::record_content(
                                on_event,
                                result,
                                c,
                                token_count,
                                first_token,
                                buffer_content,
                            );
                        }
                    }
                }
            }
            ParsedChunk::ToolCalls(tool_calls) => acc.ingest(&tool_calls),
            ParsedChunk::Usage(usage) => {
                result.eval_count = usage.output_tokens.and_then(|value| value.try_into().ok());
                result.prompt_tokens = usage.input_tokens.and_then(|value| value.try_into().ok());
                result.usage = Some(usage);
            }
        }
    }
}
