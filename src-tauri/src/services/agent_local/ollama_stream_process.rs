use super::ollama_stream_filter::emit_filtered;
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::{StreamEvent, StreamResult};
use crate::services::llm::reasoning_wire::ReasoningCapture;
use crate::services::stream_utils::ThinkTagFilter;
use tokio::sync::mpsc;

pub(crate) use super::ollama_stream_filter::flush_filter;

pub fn process_chunk(
    text: &str,
    on_event: &AgentEventEmitter,
    token_count: &mut u32,
    result: &mut StreamResult,
    tool_tx: Option<&mpsc::UnboundedSender<(usize, String, serde_json::Value)>>,
    think_filter: &mut ThinkTagFilter,
    buffer_content: bool,
    mut reasoning_capture: Option<&mut ReasoningCapture>,
) -> Result<(), String> {
    let chunk: serde_json::Value =
        serde_json::from_str(text).map_err(|e| format!("JSON invalide: {e}"))?;
    if let Some(reasoning_capture) = reasoning_capture.as_deref_mut() {
        reasoning_capture.observe_json(&chunk);
        reasoning_capture.observe_done(&chunk);
    }

    result.total_chunks = result.total_chunks.saturating_add(1);

    if let Some(err) = chunk["error"].as_str() {
        ::log::error!("[ollama-stream] erreur modèle: {err}");
        return Err(format!("Ollama: {err}"));
    }

    if chunk["done"].as_bool() == Some(true) {
        result.eval_count = chunk["eval_count"]
            .as_u64()
            .and_then(|value| value.try_into().ok());
        result.prompt_tokens = chunk["prompt_eval_count"]
            .as_u64()
            .and_then(|value| value.try_into().ok());
        result.done_reason = chunk["done_reason"].as_str().map(|s| s.to_string());
        result.total_duration_ns = chunk["total_duration"].as_u64();
        if let Some(duration_ns) = done_generation_duration(&chunk) {
            result.generation.record_native_duration(duration_ns);
        }
        flush_filter(think_filter, on_event, token_count, result, buffer_content);
        result.continuation = reasoning_capture
            .as_deref_mut()
            .and_then(ReasoningCapture::finish_complete);
        return Ok(());
    }

    let msg = &chunk["message"];

    let mut chunk_has_payload = false;

    if let Some(thinking) = msg["thinking"].as_str() {
        if !thinking.is_empty() {
            chunk_has_payload = true;
            super::stream_buffer::record_thinking(
                on_event,
                result,
                thinking.to_string(),
                token_count,
            );
        }
    }

    if let Some(content) = msg["content"].as_str() {
        if !content.is_empty() {
            chunk_has_payload = true;
            super::stream_buffer::record_generation_started(on_event, result);
            emit_filtered(
                think_filter,
                content,
                on_event,
                token_count,
                result,
                buffer_content,
            );
        }
    }

    if let Some(tool_calls) = msg["tool_calls"].as_array() {
        if !tool_calls.is_empty() {
            chunk_has_payload = true;
            super::stream_buffer::record_generation_started(on_event, result);
        }
        for tc in tool_calls {
            let func = &tc["function"];
            let name = func["name"].as_str().unwrap_or("").to_string();
            let args = func["arguments"].clone();
            let idx = result.tool_calls.len();
            // Ollama n'émet pas d'ID natif. Le journal et les résultats d'outil
            // exigent néanmoins une identité stable : elle reste locale et le
            // wire Ollama la retire avant toute requête sortante.
            let tool_call_id = uuid::Uuid::new_v4().to_string();
            super::stream_buffer::record_tool_call_generation(
                on_event,
                result,
                &name,
                &args,
                token_count,
            );
            result.tool_calls.push((name.clone(), args.clone()));
            result.tool_call_ids.push(tool_call_id.clone());
            let _ = on_event.send(StreamEvent::ToolCall {
                name: name.clone(),
                arguments: args.clone(),
                tool_call_index: idx,
                tool_call_id: Some(tool_call_id),
                domain: super::memory_tool::event_domain(&name, &args),
            });
            if let Some(tx) = tool_tx {
                let _ = tx.send((idx, name, args));
            }
        }
    }

    if !chunk_has_payload {
        result.empty_chunks = result.empty_chunks.saturating_add(1);
    }

    Ok(())
}

pub(crate) fn done_generation_duration(chunk: &serde_json::Value) -> Option<u64> {
    chunk["eval_duration"]
        .as_u64()
        .filter(|duration| super::generation_metrics::valid_duration_ns(*duration))
}
