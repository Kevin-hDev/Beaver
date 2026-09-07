use super::stream_diagnostics_support as support;
use super::types_ollama::{ChatMessage, StreamResult};
use crate::services::reasoning_continuity::envelope::ContinuationState;

#[derive(Debug, Default, PartialEq)]
struct ModelRequestStats {
    messages: usize,
    assistant_messages: usize,
    assistant_reasoning_messages: usize,
    assistant_reasoning_chars: usize,
    assistant_content_chars: usize,
    assistant_tool_calls: usize,
    tool_messages: usize,
}

pub async fn record_model_request(
    session_id: &str,
    request_id: &str,
    turn: usize,
    messages: &[ChatMessage],
) {
    let stats = request_stats(messages);
    let message = format!(
        "model_request turn={} messages={} assistant={} reasoning_msgs={} reasoning_chars={} assistant_content_chars={} tool_calls={} tool_results={}",
        turn + 1,
        stats.messages,
        stats.assistant_messages,
        stats.assistant_reasoning_messages,
        stats.assistant_reasoning_chars,
        stats.assistant_content_chars,
        stats.assistant_tool_calls,
        stats.tool_messages
    );
    record(session_id, request_id, "model_request", &message).await;
}

pub async fn record_model_result(
    session_id: &str,
    request_id: &str,
    turn: usize,
    result: &StreamResult,
) {
    let usage = result.usage.as_ref();
    let message = format!(
        "model_result turn={} content_chars={} thinking_chars={} tool_calls={} prompt_tokens={} eval_tokens={} cache_read_tokens={} cache_write_tokens={} cache_miss_tokens={} cache_status={} done_reason={} total_chunks={} empty_chunks={}",
        turn + 1,
        char_count(&result.content),
        char_count(&result.thinking),
        result.tool_calls.len(),
        opt_count(result.prompt_tokens),
        opt_count(result.eval_count),
        opt_count_u64(usage.and_then(|usage| usage.cached_input_tokens)),
        opt_count_u64(usage.and_then(|usage| usage.cache_write_input_tokens)),
        opt_count_u64(usage.and_then(|usage| usage.cache_miss_input_tokens)),
        usage
            .map(crate::services::provider_usage::RequestUsage::cache_status_label)
            .unwrap_or("unknown"),
        result.done_reason.as_deref().unwrap_or("unknown"),
        result.total_chunks,
        result.empty_chunks
    );
    record(session_id, request_id, "model_result", &message).await;
}

async fn record(session_id: &str, request_id: &str, phase: &str, message: &str) {
    let _ = support::update_run(session_id, request_id, |_session, run| {
        run.phase = phase.to_string();
        run.safe_summary = Some(message.to_string());
        support::push_event(run, phase, message, None, None);
    })
    .await;
}

fn request_stats(messages: &[ChatMessage]) -> ModelRequestStats {
    let mut stats = ModelRequestStats {
        messages: messages.len(),
        ..Default::default()
    };
    for message in messages {
        match message.role.as_str() {
            "assistant" => {
                stats.assistant_messages += 1;
                stats.assistant_content_chars += char_count(&message.content);
                stats.assistant_tool_calls += message.tool_calls.as_ref().map_or(0, Vec::len);
                let native_reasoning = message.continuation.as_ref().and_then(|envelope| {
                    match &envelope.continuation {
                        ContinuationState::OllamaNative { thinking } => Some(thinking),
                        _ => None,
                    }
                });
                if let Some(reasoning) = native_reasoning.or(message.tool_loop_reasoning.as_ref()) {
                    if !reasoning.is_empty() {
                        stats.assistant_reasoning_messages += 1;
                        stats.assistant_reasoning_chars += char_count(reasoning);
                    }
                }
            }
            "tool" => stats.tool_messages += 1,
            _ => {}
        }
    }
    stats
}

fn char_count(value: &str) -> usize {
    value.chars().count()
}

fn opt_count(value: Option<u32>) -> String {
    value
        .map(|count| count.to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn opt_count_u64(value: Option<u64>) -> String {
    value
        .map(|count| count.to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

#[cfg(test)]
#[path = "stream_diagnostics_model_tests.rs"]
mod tests;
