use crate::services::agent_local::ollama_stream_process::{flush_filter, process_chunk};
use crate::services::agent_local::ollama_stream_request::{
    open_chat_response, OpenChatResponse, RetryCounts,
};
use crate::services::agent_local::ollama_tool_parse_retry::{
    is_tool_parse_crash, MAX_PARSER_RETRIES,
};
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::{
    ChatRequest, StreamEvent, StreamOutcome, StreamResult,
};
use crate::services::compress::realtime_budget::RealtimeBudget;
use crate::services::stream_utils::ThinkTagFilter;
use futures_util::StreamExt;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;
use tokio_util::io::StreamReader;
use tokio_util::sync::CancellationToken;

pub use crate::services::agent_local::ollama_collect::{
    collect_chat, collect_chat_with_timeout_and_limit,
};

/// Variante avec eager dispatch : les tool calls sont envoyés via `tool_tx` dès réception.
pub async fn stream_chat_with_tool_notify(
    on_event: &AgentEventEmitter,
    request: &ChatRequest,
    cancel: CancellationToken,
    tool_tx: mpsc::UnboundedSender<(usize, String, serde_json::Value)>,
    buffer_content: bool,
    realtime_budget: Option<RealtimeBudget>,
) -> Result<StreamOutcome, String> {
    stream_chat_inner(
        on_event,
        request,
        cancel,
        false,
        Some(tool_tx),
        buffer_content,
        realtime_budget,
        RetryCounts {
            parser_retries: 0,
            server_retries: 0,
        },
    )
    .await
}

async fn stream_chat_inner(
    on_event: &AgentEventEmitter,
    request: &ChatRequest,
    cancel: CancellationToken,
    emit_done: bool,
    tool_tx: Option<mpsc::UnboundedSender<(usize, String, serde_json::Value)>>,
    buffer_content: bool,
    mut realtime_budget: Option<RealtimeBudget>,
    retry_counts: RetryCounts,
) -> Result<StreamOutcome, String> {
    let resp = match open_chat_response(on_event, request, &cancel, retry_counts, !buffer_content)
        .await?
    {
        OpenChatResponse::Ready(response) => response,
        OpenChatResponse::Retry { request, counts } => {
            return Box::pin(stream_chat_inner(
                on_event,
                &request,
                cancel,
                emit_done,
                tool_tx,
                buffer_content,
                realtime_budget,
                counts,
            ))
            .await;
        }
    };

    let http_status = resp.status();
    let byte_stream = resp
        .bytes_stream()
        .map(|r| r.map_err(std::io::Error::other));
    let mut lines = BufReader::new(StreamReader::new(byte_stream)).lines();

    eprintln!(
        "[ollama-stream] stream ouvert HTTP {} model={} think={:?} msgs={} tools={}",
        http_status,
        request.model,
        request.think,
        request.messages.len(),
        request.tools.as_ref().map_or(0, Vec::len)
    );

    let mut token_count: u32 = 0;
    let mut first_token: Option<std::time::Instant> = None;
    let mut result = StreamResult::default();
    let mut think_filter = ThinkTagFilter::new();
    let mut interrupted = false;

    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                return Err("Annulé".to_string());
            }
            _ = tokio::time::sleep(std::time::Duration::from_secs(300)) => {
                let msg = "Timeout : aucune réponse d'Ollama depuis 5 min".to_string();
                let _ = on_event.send(StreamEvent::Error { message: msg.clone(), is_connection: false, diagnostic: None });
                return Err(msg);
            }
            line = lines.next_line() => {
                match line {
                    Ok(Some(text)) => {
                        if let Err(e) = process_chunk(
                            &text, on_event, &mut token_count, &mut first_token,
                            &mut result, emit_done, tool_tx.as_ref(), &mut think_filter,
                            buffer_content,
                        ) {
                            // Bug Ollama #16383 : crash du parser tool-call en plein
                            // stream. Si aucun contenu final n'a encore été émis (on
                            // n'a reçu que du thinking), on peut retenter proprement.
                            if is_tool_parse_crash(&e)
                                && retry_counts.parser_retries < MAX_PARSER_RETRIES
                                && result.content.is_empty()
                            {
                                let attempt = retry_counts.parser_retries + 1;
                                eprintln!(
                                    "[ollama-stream] crash parser tool-call mid-stream (#{}), retry",
                                    attempt
                                );
                                if !buffer_content {
                                    crate::services::agent_local::ollama_retry_indicator::send_retry_indicator(
                                        on_event,
                                        crate::services::agent_local::ollama_retry_indicator::REASON_PARSER_CRASH,
                                        attempt,
                                        MAX_PARSER_RETRIES,
                                    );
                                }
                                return Box::pin(stream_chat_inner(
                                    on_event,
                                    request,
                                    cancel,
                                    emit_done,
                                    tool_tx,
                                    buffer_content,
                                    realtime_budget,
                                    RetryCounts {
                                        parser_retries: attempt,
                                        ..retry_counts
                                    },
                                ))
                                .await;
                            }
                            let _ = on_event.send(StreamEvent::Error { message: e.clone(), is_connection: false, diagnostic: None });
                            return Err(e);
                        }
                        if should_interrupt(&mut realtime_budget, token_count, !result.tool_calls.is_empty()) {
                            interrupted = true;
                            break;
                        }
                    }
                    Ok(None) => break,
                    Err(e) => {
                        let is_conn = e.kind() == std::io::ErrorKind::ConnectionReset
                            || e.kind() == std::io::ErrorKind::ConnectionAborted
                            || e.kind() == std::io::ErrorKind::UnexpectedEof
                            || e.to_string().contains("decoding");
                        let msg = "ollama_connection_lost".to_string();
                        let _ = on_event.send(StreamEvent::Error { message: msg.clone(), is_connection: is_conn, diagnostic: None });
                        return Err(msg);
                    }
                }
            }
        }
    }
    if interrupted {
        flush_filter(
            &mut think_filter,
            on_event,
            &mut token_count,
            &mut first_token,
            &mut result,
            buffer_content,
        );
        eprintln!(
            "[ollama-stream] fin=interrupted content_chars={} thinking_chars={} tool_calls={} done_reason={} chunks={} empty_chunks={}",
            result.content.chars().count(),
            result.thinking.chars().count(),
            result.tool_calls.len(),
            result.done_reason.as_deref().unwrap_or("none"),
            result.total_chunks,
            result.empty_chunks
        );
        Ok(StreamOutcome::InterruptedForCompression(result))
    } else {
        eprintln!(
            "[ollama-stream] fin=eof content_chars={} thinking_chars={} tool_calls={} done_reason={} chunks={} empty_chunks={}",
            result.content.chars().count(),
            result.thinking.chars().count(),
            result.tool_calls.len(),
            result.done_reason.as_deref().unwrap_or("none"),
            result.total_chunks,
            result.empty_chunks
        );
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
