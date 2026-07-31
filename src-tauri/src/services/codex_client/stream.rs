use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::{
    ChatMessage, StreamEvent, StreamOutcome, StreamResult,
};
use crate::services::compress::realtime_budget::RealtimeBudget;
use eventsource_stream::Eventsource;
use futures_util::StreamExt;
use tokio_util::sync::CancellationToken;

use super::{http_error, replay::ReplayCollector, request};

pub const CODEX_IDLE_TIMEOUT_SECS: u64 = 180;

pub use super::stream_silent::{collect_chat_silent, collect_chat_silent_for_compression};

pub async fn stream_chat_with_budget(
    on_event: &AgentEventEmitter,
    model: &str,
    messages: &[ChatMessage],
    tools: &[serde_json::Value],
    think: bool,
    reasoning_mode: Option<&str>,
    cancel: CancellationToken,
    buffer_content: bool,
    realtime_budget: Option<RealtimeBudget>,
) -> Result<StreamOutcome, String> {
    let resp = request::post_codex_stream(model, messages, tools, think, reasoning_mode).await?;
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
    on_event: &AgentEventEmitter,
    resp: reqwest::Response,
    cancel: CancellationToken,
    buffer_content: bool,
    mut realtime_budget: Option<RealtimeBudget>,
    tools: &[serde_json::Value],
) -> Result<StreamOutcome, String> {
    let mut sse = resp.bytes_stream().eventsource();
    let mut result = StreamResult::default();
    let mut token_count: u32 = 0;
    let mut cur_tool_id: Option<String> = None;
    let mut cur_tool_name: Option<String> = None;
    let mut cur_tool_args = String::new();
    let mut interrupted = false;
    let mut replay = ReplayCollector::default();
    loop {
        let event = tokio::select! {
            _ = cancel.cancelled() => return Err("Annulé".to_string()),
            _ = tokio::time::sleep(std::time::Duration::from_secs(CODEX_IDLE_TIMEOUT_SECS)) => {
                return Err("provider_temporarily_unavailable".to_string());
            }
            ev = sse.next() => match ev {
                Some(Ok(e)) => e,
                Some(Err(_)) => return Err("provider_connection_failed".to_string()),
                None => break,
            },
        };

        if event.data.trim() == "[DONE]" {
            break;
        }
        let parsed: serde_json::Value = match serde_json::from_str(&event.data) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let etype = parsed["type"].as_str().unwrap_or("");

        match etype {
            "response.reasoning_summary_text.delta" => {
                let delta = parsed["delta"].as_str().unwrap_or("");
                if !delta.is_empty() {
                    crate::services::agent_local::stream_buffer::record_thinking(
                        on_event,
                        &mut result,
                        delta.to_string(),
                        &mut token_count,
                    );
                }
            }
            "response.output_text.delta" => {
                let delta = parsed["delta"].as_str().unwrap_or("");
                if delta.is_empty() {
                    continue;
                }
                crate::services::agent_local::stream_buffer::record_content(
                    on_event,
                    &mut result,
                    delta.to_string(),
                    &mut token_count,
                    buffer_content,
                );
                if should_interrupt(
                    &mut realtime_budget,
                    token_count,
                    cur_tool_id.is_some() || !result.tool_calls.is_empty(),
                ) {
                    interrupted = true;
                    break;
                }
            }
            "response.output_item.added" => {
                if let Some(item) = parsed.get("item") {
                    if item["type"].as_str() == Some("function_call") {
                        crate::services::agent_local::stream_buffer::record_generation_started(
                            on_event,
                            &mut result,
                        );
                        cur_tool_id = item["call_id"].as_str().map(String::from);
                        cur_tool_name = item["name"].as_str().map(|name| {
                            crate::services::llm::tool_schema::restore_tool_name(name, tools)
                        });
                        cur_tool_args.clear();
                    }
                }
            }
            "response.function_call_arguments.delta" => {
                let delta = parsed["delta"].as_str().unwrap_or("");
                if !delta.is_empty() {
                    crate::services::agent_local::stream_buffer::record_generation_started(
                        on_event,
                        &mut result,
                    );
                }
                cur_tool_args.push_str(delta);
            }
            "response.output_item.done" => {
                if let Some(item) = parsed.get("item") {
                    let mut replay_item = item.clone();
                    super::replay::restore_tool_name(&mut replay_item, tools);
                    replay.capture(&replay_item)?;
                }
                if let (Some(id), Some(name)) = (cur_tool_id.take(), cur_tool_name.take()) {
                    let args_json: serde_json::Value =
                        serde_json::from_str(&cur_tool_args).unwrap_or_default();
                    crate::services::agent_local::stream_buffer::record_tool_call_generation(
                        on_event,
                        &mut result,
                        &name,
                        &args_json,
                        &mut token_count,
                    );
                    let _ = on_event.send(StreamEvent::ToolCall {
                        name: name.clone(),
                        arguments: args_json.clone(),
                        domain: crate::services::agent_local::memory_tool::event_domain(
                            &name, &args_json,
                        ),
                    });
                    result.tool_calls.push((name, args_json));
                    result.tool_call_ids.push(id);
                    cur_tool_args.clear();
                }
            }
            "response.done" | "response.completed" => {
                if let Some(usage) = parsed.pointer("/response/usage") {
                    result.usage = crate::services::provider_usage::RequestUsage::from_json(usage);
                    if let Some(usage) = &result.usage {
                        result.prompt_tokens =
                            usage.input_tokens.and_then(|value| value.try_into().ok());
                        result.eval_count =
                            usage.output_tokens.and_then(|value| value.try_into().ok());
                    }
                }
                break;
            }
            "response.failed" => {
                return Err(http_error::stream_failure(&parsed));
            }
            _ => {}
        }
    }

    replay.attach(&mut result);
    if interrupted {
        Ok(StreamOutcome::InterruptedForCompression(result))
    } else {
        Ok(StreamOutcome::Completed(result))
    }
}

fn should_interrupt(
    budget: &mut Option<RealtimeBudget>,
    token_count: u32,
    has_tool_call: bool,
) -> bool {
    !has_tool_call
        && budget
            .as_mut()
            .is_some_and(|budget| budget.should_interrupt(token_count))
}
