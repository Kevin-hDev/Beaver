use super::{
    stream_chunk::{self, ParsedChunk},
    stream_http::{post_chat_request, post_chat_request_with_timeout, RequestConfig},
    stream_sse::is_done_marker,
    stream_tools::ToolCallAccumulator,
};
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::agent_local::types_ollama::StreamResult;
use crate::services::llm::request_purpose::RequestPurpose;
use crate::services::stream_utils::{FilteredChunk, ThinkTagFilter};
use eventsource_stream::Eventsource;
use futures_util::StreamExt;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

pub async fn collect_chat_silent(
    provider_id: &str,
    model: &str,
    messages: &[ChatMessage],
    purpose: RequestPurpose,
    cancel: CancellationToken,
) -> Result<StreamResult, String> {
    if provider_id == "codex-oauth" {
        return crate::services::codex_client::stream::collect_chat_silent(
            model,
            messages,
            &[],
            false,
            None,
            None,
            cancel,
        )
        .await;
    }
    let cfg = RequestConfig {
        provider_id,
        model,
        messages,
        tools: &[],
        think: false,
        reasoning_mode: None,
        max_tokens: None,
        purpose,
    };
    let resp = post_chat_request(&cfg).await.map_err(|e| e.to_string())?;
    consume_silent(resp, cancel, super::timeouts::idle_timeout()).await
}

pub async fn collect_chat_silent_for_compression(
    provider_id: &str,
    model: &str,
    messages: &[ChatMessage],
    max_tokens: u32,
    purpose: RequestPurpose,
    cancel: CancellationToken,
) -> Result<StreamResult, String> {
    let request_timeout = crate::services::compress::timeouts::compression_request_timeout();
    let idle_timeout = crate::services::compress::timeouts::compression_idle_timeout();
    if provider_id == "codex-oauth" {
        return crate::services::codex_client::stream::collect_chat_silent_for_compression(
            model,
            messages,
            &[],
            false,
            None,
            Some(max_tokens),
            cancel,
        )
        .await;
    }
    let cfg = RequestConfig {
        provider_id,
        model,
        messages,
        tools: &[],
        think: false,
        reasoning_mode: None,
        max_tokens: Some(max_tokens),
        purpose,
    };
    let resp = post_chat_request_with_timeout(&cfg, request_timeout)
        .await
        .map_err(|e| e.to_string())?;
    consume_silent(resp, cancel, idle_timeout).await
}

async fn consume_silent(
    resp: reqwest::Response,
    cancel: CancellationToken,
    idle_timeout: Duration,
) -> Result<StreamResult, String> {
    let mut stream = resp.bytes_stream().eventsource();
    let mut result = StreamResult::default();
    let mut acc = ToolCallAccumulator::new();
    let mut think_filter = ThinkTagFilter::new();

    loop {
        tokio::select! {
            _ = cancel.cancelled() => return Err("Annulé".to_string()),
            _ = tokio::time::sleep(idle_timeout) => {
                return Err(format!(
                    "Timeout compression : aucune réponse depuis {}s",
                    idle_timeout.as_secs()
                ));
            }
            event = stream.next() => {
                let Some(event) = event else { break; };
                let event = event.map_err(|e| format!("SSE: {e}"))?;
                if is_done_marker(&event.data) { break; }
                process_chunk_silent(&event.data, &mut result, &mut acc, &mut think_filter);
            }
        }
    }

    for chunk in think_filter.flush() {
        if let FilteredChunk::Content(c) = chunk {
            result.content.push_str(&c);
        }
    }

    let (tool_calls, ids, extra_content) = acc.finalize();
    for (i, (name, args)) in tool_calls.iter().enumerate() {
        result.tool_calls.push((name.clone(), args.clone()));
        if let Some(id) = ids.get(i) {
            result.tool_call_ids.push(id.clone());
        }
        result
            .tool_call_extra_content
            .push(extra_content.get(i).cloned().flatten());
    }

    Ok(result)
}

fn process_chunk_silent(
    data: &str,
    result: &mut StreamResult,
    acc: &mut ToolCallAccumulator,
    think_filter: &mut ThinkTagFilter,
) {
    for chunk in stream_chunk::parse(data) {
        match chunk {
            ParsedChunk::Content(content) => {
                for filtered in think_filter.feed(&content) {
                    if let FilteredChunk::Content(c) = filtered {
                        result.content.push_str(&c);
                    }
                }
            }
            ParsedChunk::Thinking(_) => {}
            ParsedChunk::ToolCalls(tool_calls) => acc.ingest(&tool_calls),
            ParsedChunk::Usage(usage) => {
                result.eval_count = usage.output_tokens.and_then(|value| value.try_into().ok());
                result.prompt_tokens = usage.input_tokens.and_then(|value| value.try_into().ok());
                result.usage = Some(usage);
            }
        }
    }
}
