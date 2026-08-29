#[cfg(test)]
use super::stream_state::ConsumedStream;
use super::stream_state::StreamState;
use crate::services::provider_usage::UsageContext;

#[allow(
    clippy::too_many_arguments,
    reason = "stream lifecycle dependencies stay explicit"
)]
pub(in crate::services::llm) async fn consume_stream(
    on_event: &crate::services::agent_local::stream_events::AgentEventEmitter,
    response: reqwest::Response,
    cancel: tokio_util::sync::CancellationToken,
    buffer_content: bool,
    mut realtime_budget: Option<crate::services::compress::realtime_budget::RealtimeBudget>,
    tools: &[serde_json::Value],
    usage_context: UsageContext<'_>,
    mut reasoning_capture: Option<crate::services::llm::reasoning_wire::ReasoningCapture>,
    mut measurement: Option<&mut crate::services::provider_usage::RequestMeasurement>,
) -> Result<crate::services::agent_local::types_ollama::StreamOutcome, String> {
    use eventsource_stream::Eventsource;
    use futures_util::StreamExt;

    if let Some(value) = response
        .headers()
        .get("request-id")
        .and_then(|value| value.to_str().ok())
    {
        if let Some(measurement) = measurement.as_mut() {
            measurement.observe_provider_request_id(value);
        }
    }
    let mut events = response.bytes_stream().eventsource();
    let mut state = StreamState::default();
    let mut result = crate::services::agent_local::types_ollama::StreamResult::default();
    let mut token_count = 0;
    let mut interrupted = false;
    loop {
        tokio::select! {
            biased;
            _ = cancel.cancelled() => return Err("Annulé".into()),
            _ = tokio::time::sleep(super::super::timeouts::idle_timeout_for(
                usage_context.canonical_provider_id,
            )) => return Err("provider_temporarily_unavailable".into()),
            event = events.next() => {
                let event = event.ok_or_else(|| "provider_connection_failed".to_string())?
                    .map_err(|_| "provider_connection_failed".to_string())?;
                let value = crate::services::llm::stream_sse::parse_json(&event.data)?;
                if let Some(measurement) = measurement.as_mut() {
                    measurement.mark_first_event();
                }
                let content_start = state.content.len();
                let thinking_start = state.thinking.chars().count();
                let tool_start = state.tool_calls.len();
                state.apply(&value, usage_context)?;
                if state.thinking.chars().count() > thinking_start {
                    super::stream_record::thinking(
                        on_event, &mut result, &state, thinking_start, &mut token_count,
                    );
                }
                if state.content.len() > content_start {
                    let delta = state.content[content_start..].to_string();
                    crate::services::agent_local::stream_buffer::record_content(
                        on_event, &mut result, delta, &mut token_count, buffer_content,
                    );
                    if let Some(measurement) = measurement.as_mut() {
                        measurement.mark_first_useful();
                    }
                }
                for index in tool_start..state.tool_calls.len() {
                    super::stream_record::tool(
                        on_event, &mut result, &state, tools, index, &mut token_count,
                    );
                    if let Some(measurement) = measurement.as_mut() {
                        measurement.mark_first_useful();
                    }
                }
                if let Some(reason) = state.finish_reason.as_deref() {
                    if let Some(measurement) = measurement.as_mut() {
                        measurement.observe_finish_reason(reason);
                    }
                }
                if value.get("type").and_then(serde_json::Value::as_str) == Some("message_stop") {
                    break;
                }
                if crate::services::llm::stream_consume_budget::should_interrupt(
                    &mut realtime_budget,
                    token_count,
                    state.has_pending_tool(),
                ) {
                    interrupted = true;
                    break;
                }
            }
        }
    }
    let consumed = if interrupted {
        state.finish_partial()?
    } else {
        state.finish()?
    };
    if !interrupted {
        if let Some(capture) = reasoning_capture.as_mut() {
            for block in &consumed.continuation_blocks {
                capture.observe_anthropic_block(block.clone());
            }
            capture.observe_persisted_tool_links(&result.tool_calls, &result.tool_call_ids);
            capture.observe_done(&serde_json::json!({"type": "message_stop"}));
        }
    }
    result.prompt_tokens = consumed
        .usage
        .as_ref()
        .and_then(|usage| usage.input_tokens)
        .and_then(|value| value.try_into().ok());
    result.eval_count = consumed
        .usage
        .as_ref()
        .and_then(|usage| usage.output_tokens)
        .and_then(|value| value.try_into().ok());
    result.usage = consumed.usage;
    result.continuation = reasoning_capture.and_then(|mut capture| {
        if interrupted {
            capture.finish_partial()
        } else {
            capture.finish_complete()
        }
    });
    Ok(if interrupted {
        crate::services::agent_local::types_ollama::StreamOutcome::InterruptedForCompression(result)
    } else {
        crate::services::agent_local::types_ollama::StreamOutcome::Completed(result)
    })
}

pub(super) async fn consume_silent(
    response: reqwest::Response,
    cancel: tokio_util::sync::CancellationToken,
    usage_context: UsageContext<'_>,
    mut measurement: Option<&mut crate::services::provider_usage::RequestMeasurement>,
) -> Result<crate::services::agent_local::types_ollama::StreamResult, String> {
    use eventsource_stream::Eventsource;
    use futures_util::StreamExt;

    let mut events = response.bytes_stream().eventsource();
    let mut state = StreamState::default();
    loop {
        tokio::select! {
            biased;
            _ = cancel.cancelled() => return Err("Annulé".into()),
            _ = tokio::time::sleep(super::super::timeouts::idle_timeout_for(
                usage_context.canonical_provider_id,
            )) => {
                return Err("provider_temporarily_unavailable".into());
            }
            event = events.next() => {
                let event = event.ok_or_else(|| "provider_connection_failed".to_string())?
                    .map_err(|_| "provider_connection_failed".to_string())?;
                let value = crate::services::llm::stream_sse::parse_json(&event.data)?;
                if let Some(measurement) = measurement.as_mut() {
                    measurement.mark_first_event();
                }
                state.apply(&value, usage_context)?;
                if value.get("type").and_then(serde_json::Value::as_str) == Some("message_stop") {
                    break;
                }
            }
        }
    }
    let consumed = state.finish()?;
    Ok(crate::services::agent_local::types_ollama::StreamResult {
        content: consumed.content,
        thinking: consumed.thinking,
        prompt_tokens: consumed
            .usage
            .as_ref()
            .and_then(|usage| usage.input_tokens)
            .and_then(|value| value.try_into().ok()),
        eval_count: consumed
            .usage
            .as_ref()
            .and_then(|usage| usage.output_tokens)
            .and_then(|value| value.try_into().ok()),
        usage: consumed.usage,
        ..Default::default()
    })
}

#[cfg(test)]
pub(super) fn consume_fixture(
    input: &str,
    context: UsageContext<'_>,
) -> Result<ConsumedStream, String> {
    let mut state = StreamState::default();
    for line in input.lines() {
        let Some(data) = line.strip_prefix("data:") else {
            continue;
        };
        let value = crate::services::llm::stream_sse::parse_json(data.trim())?;
        state.apply(&value, context)?;
    }
    state.finish()
}
