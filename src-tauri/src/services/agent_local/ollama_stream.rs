use crate::services::agent_local::ollama_client::OllamaClient;
use crate::services::agent_local::ollama_stream_process::{flush_filter, process_chunk};
use crate::services::agent_local::ollama_stream_request::{
    open_chat_response, OpenChatResponse, ReplayDiagnosticContext, RetryCounts, StreamChatOptions,
};
use crate::services::agent_local::ollama_tool_parse_retry::{
    is_tool_parse_crash, MAX_PARSER_RETRIES,
};
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::{
    ChatRequest, StreamEvent, StreamOutcome, StreamResult,
};
use crate::services::compress::realtime_budget::RealtimeBudget;
use crate::services::llm::reasoning_wire::{ReasoningCapture, ReasoningCaptureContext};
use crate::services::reasoning_continuity::contract::{CredentialScope, RouteId};
use crate::services::stream_utils::ThinkTagFilter;
use futures_util::StreamExt;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;
use tokio_util::io::StreamReader;
use tokio_util::sync::CancellationToken;

/// Variante avec eager dispatch : les tool calls sont envoyés via `tool_tx` dès réception.
pub async fn stream_chat_with_tool_notify(
    on_event: &AgentEventEmitter,
    request: &ChatRequest,
    cancel: CancellationToken,
    tool_tx: mpsc::UnboundedSender<(usize, String, serde_json::Value)>,
    buffer_content: bool,
    realtime_budget: Option<RealtimeBudget>,
    diagnostics: ReplayDiagnosticContext<'_>,
) -> Result<StreamOutcome, String> {
    let ollama = OllamaClient::from_global()?;
    stream_chat_inner(
        &ollama,
        on_event,
        request,
        cancel,
        diagnostics,
        StreamChatOptions {
            tool_tx: Some(tool_tx),
            buffer_content,
            realtime_budget,
            retry_counts: RetryCounts {
                parser_retries: 0,
                server_retries: 0,
            },
        },
    )
    .await
}

async fn stream_chat_inner(
    ollama: &OllamaClient,
    on_event: &AgentEventEmitter,
    request: &ChatRequest,
    cancel: CancellationToken,
    diagnostics: ReplayDiagnosticContext<'_>,
    mut options: StreamChatOptions,
) -> Result<StreamOutcome, String> {
    let resp = match open_chat_response(
        ollama,
        on_event,
        request,
        &cancel,
        options.retry_counts,
        !options.buffer_content,
        diagnostics,
    )
    .await?
    {
        OpenChatResponse::Ready(response) => response,
        OpenChatResponse::Retry { request, counts } => {
            return Box::pin(stream_chat_inner(
                ollama,
                on_event,
                &request,
                cancel,
                diagnostics,
                StreamChatOptions {
                    retry_counts: counts,
                    ..options
                },
            ))
            .await;
        }
    };

    let http_status = resp.status();
    let byte_stream = resp
        .bytes_stream()
        .map(|r| r.map_err(std::io::Error::other));
    let mut lines = BufReader::new(StreamReader::new(byte_stream)).lines();

    ::log::info!(
        "[ollama-stream] stream ouvert HTTP {} model={} think={:?} msgs={} tools={}",
        http_status,
        request.model,
        request.think,
        request.messages.len(),
        request.tools.as_ref().map_or(0, Vec::len)
    );

    let mut token_count: u32 = 0;
    let mut result = StreamResult::default();
    let mut reasoning_capture = request
        .capture_reasoning
        .then(|| {
            ReasoningCapture::new(ReasoningCaptureContext {
                route_id: RouteId::Ollama,
                model_id: request.model.clone(),
                credential_scope: CredentialScope::local_uncredentialed(),
                reasoning_mode: super::ollama_stream_policy::reasoning_mode(request),
            })
        })
        .transpose()
        .map_err(|_| "provider_configuration_invalid".to_string())?;
    let mut think_filter = ThinkTagFilter::new();
    let mut fragments = crate::services::llm::stream_fragments::StreamFragmentState::ollama();
    let mut interrupted = false;

    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                return Err("Annulé".to_string());
            }
            _ = tokio::time::sleep(std::time::Duration::from_secs(300)) => {
                let msg = "Timeout : aucune réponse d'Ollama depuis 5 min".to_string();
                let _ = on_event.send(StreamEvent::Error { message: msg.clone(), is_connection: false, context_capacity: None, diagnostic: None });
                return Err(msg);
            }
            line = lines.next_line() => {
                match line {
                    Ok(Some(text)) => {
                        if let Err(e) = process_chunk(
                            &text, on_event, &mut token_count,
                            &mut result, options.tool_tx.as_ref(), &mut think_filter,
                            super::ollama_stream_process::ProcessChunkOptions {
                                buffer_content: options.buffer_content,
                                reasoning_capture: reasoning_capture.as_mut(),
                                fragments: &mut fragments,
                            },
                        ) {
                            // Bug Ollama #16383 : crash du parser tool-call en plein
                            // stream. Si aucun contenu final n'a encore été émis (on
                            // n'a reçu que du thinking), on peut retenter proprement.
                            if is_tool_parse_crash(&e)
                                && options.retry_counts.parser_retries < MAX_PARSER_RETRIES
                                && result.content.is_empty()
                            {
                                let attempt = options.retry_counts.parser_retries + 1;
                                ::log::warn!(
                                    "[ollama-stream] crash parser tool-call mid-stream (#{}), retry",
                                    attempt
                                );
                                if !options.buffer_content {
                                    crate::services::agent_local::ollama_retry_indicator::send_retry_indicator(
                                        on_event,
                                        crate::services::agent_local::ollama_retry_indicator::REASON_PARSER_CRASH,
                                        attempt,
                                        MAX_PARSER_RETRIES,
                                    );
                                }
                                return Box::pin(stream_chat_inner(
                                    ollama,
                                    on_event,
                                    request,
                                    cancel,
                                    diagnostics,
                                    StreamChatOptions {
                                        tool_tx: options.tool_tx,
                                        buffer_content: options.buffer_content,
                                        realtime_budget: options.realtime_budget,
                                        retry_counts: RetryCounts {
                                            parser_retries: attempt,
                                            ..options.retry_counts
                                        },
                                    },
                                ))
                                .await;
                            }
                            let _ = on_event.send(StreamEvent::Error { message: e.clone(), is_connection: false, context_capacity: None, diagnostic: None });
                            return Err(e);
                        }
                        if super::ollama_stream_policy::should_interrupt(
                            &mut options.realtime_budget,
                            token_count,
                            !result.tool_calls.is_empty(),
                        ) {
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
                        let _ = on_event.send(StreamEvent::Error { message: msg.clone(), is_connection: is_conn, context_capacity: None, diagnostic: None });
                        return Err(msg);
                    }
                }
            }
        }
    }
    if interrupted {
        if let Some(capture) = reasoning_capture.as_mut() {
            capture.finish_partial();
        }
        flush_filter(
            &mut think_filter,
            on_event,
            &mut token_count,
            &mut result,
            options.buffer_content,
        );
    }
    Ok(if interrupted {
        StreamOutcome::InterruptedForCompression(result)
    } else {
        StreamOutcome::Completed(result)
    })
}
