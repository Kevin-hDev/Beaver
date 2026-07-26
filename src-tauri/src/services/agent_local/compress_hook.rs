use crate::services::agent_local::ollama_stream;
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::{ChatMessage, StreamEvent};
use crate::services::compress::{engine, prompt, state, summary_budget, token_estimate};
use std::path::Path;
use tokio_util::sync::CancellationToken;

pub async fn try_auto_compress(
    on_event: &AgentEventEmitter,
    messages: &mut Vec<ChatMessage>,
    model: &str,
    session_id: &str,
    request_id: &str,
    native_context: u64,
    configured_context: u64,
    last_context_tokens: Option<u32>,
    working_dir: &Path,
    cancel: CancellationToken,
) -> Option<u32> {
    let config = match crate::services::config::read_config() {
        Ok(c) => c.advanced,
        Err(_) => return None,
    };
    let estimated = token_estimate::estimate_tokens(messages);
    let used = state::context_used_for_compression(last_context_tokens, estimated);
    if !state::is_safe_to_compress(messages) {
        return None;
    }
    if !engine::should_auto_compress(
        config.compression_enabled,
        native_context,
        configured_context,
        used,
        config.compression_threshold,
    ) {
        return None;
    }

    let _ = on_event.send(StreamEvent::Compressing {
        status: "start".to_string(),
    });
    super::stream_diagnostics::mark_phase(
        session_id,
        request_id,
        "compression",
        "Auto-compression du contexte démarrée.",
    )
    .await;

    let (summary_instruction, output_limit) =
        summary_budget::summary_instruction_for_input(configured_context, estimated);
    let compress_msgs =
        engine::build_compression_request_content(messages, summary_instruction.as_deref());
    eprintln!(
        "[compress] auto ollama start session={session_id} input_tokens={estimated} output_limit={output_limit}"
    );
    let compression = ollama_stream::collect_chat_with_timeout_and_limit(
        model,
        compress_msgs,
        crate::services::compress::timeouts::compression_request_timeout(),
        Some(output_limit),
    );
    match tokio::select! {
        _ = cancel.cancelled() => Err("Annulé".to_string()),
        result = compression => result,
    } {
        Ok((content, _)) => {
            let summary = prompt::extract_summary(&content);
            let mode = state::CompressionMode::Auto {
                request_start_index: state::request_start_index(messages),
            };
            let current_tokens = state::apply_and_save(
                session_id,
                messages,
                &summary,
                configured_context,
                true,
                working_dir,
                mode,
            )
            .await
            .unwrap_or_else(|err| {
                eprintln!("[compress] save session failed: {err}");
                token_estimate::estimate_tokens(messages) as u32
            });
            send_compression_done(on_event);
            eprintln!(
                "[compress] auto ollama done session={session_id} context_tokens={current_tokens}"
            );
            Some(current_tokens)
        }
        Err(e) => {
            if !cancel.is_cancelled() {
                eprintln!("[compress] Échec compression Ollama : {e}");
            }
            send_compressing_done(on_event);
            None
        }
    }
}

fn send_compression_done(on_event: &AgentEventEmitter) {
    send_compressing_done(on_event);
    let _ = on_event.send(StreamEvent::CompressionComplete {});
}

fn send_compressing_done(on_event: &AgentEventEmitter) {
    let _ = on_event.send(StreamEvent::Compressing {
        status: "done".to_string(),
    });
}

#[cfg(test)]
#[path = "compress_hook_tests.rs"]
mod tests;
