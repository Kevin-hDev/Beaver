use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::{ChatMessage, StreamEvent};
use std::path::Path;
use tokio_util::sync::CancellationToken;

pub(crate) fn is_compress_command(messages: &[ChatMessage]) -> bool {
    messages
        .last()
        .map(|m| m.role == "user" && m.content.trim() == "/compress")
        .unwrap_or(false)
}

pub(crate) async fn handle_compress_command(
    on_event: &AgentEventEmitter,
    session_id: &str,
    request_id: &str,
    messages: &[ChatMessage],
    model: &str,
    provider: &str,
    working_dir: &Path,
    cancel: CancellationToken,
) -> Result<(), String> {
    use crate::services::compress::{engine, prompt, state};

    let _ = on_event.send(StreamEvent::Compressing {
        status: "start".to_string(),
    });
    crate::services::agent_local::stream_diagnostics::mark_phase(
        session_id,
        request_id,
        "compression",
        "Compression du contexte démarrée.",
    )
    .await;

    let mut msgs_without_command: Vec<ChatMessage> = messages
        .iter()
        .filter(|m| !(m.role == "user" && m.content.trim() == "/compress"))
        .cloned()
        .collect();

    let input_tokens =
        crate::services::compress::token_estimate::estimate_tokens(&msgs_without_command);
    let context = resolve_context_window(provider, model).await;
    let (summary_instruction, output_limit) =
        crate::services::compress::summary_budget::summary_instruction_for_input(
            context,
            input_tokens,
        );
    eprintln!(
        "[compress] manual start session={session_id} provider={provider} input_tokens={input_tokens} output_limit={output_limit}"
    );

    let compress_msgs = engine::build_compression_request_content(
        &msgs_without_command,
        summary_instruction.as_deref(),
    );
    let summary_raw = match collect_summary(
        provider,
        model,
        session_id,
        compress_msgs,
        output_limit,
        cancel,
    )
    .await
    {
        Ok(summary) => summary,
        Err(err) => {
            eprintln!("[compress] manual failed session={session_id}: {err}");
            send_compressing_done(on_event);
            return Err(err);
        }
    };
    let summary = prompt::extract_summary(&summary_raw);
    let current_tokens = state::apply_and_save(
        session_id,
        &mut msgs_without_command,
        &summary,
        context,
        false,
        working_dir,
        state::CompressionMode::Manual,
    )
    .await?;

    send_compression_done(on_event);
    eprintln!("[compress] manual done session={session_id} context_tokens={current_tokens}");
    Ok(())
}

async fn collect_summary(
    provider: &str,
    model: &str,
    session_id: &str,
    messages: Vec<ChatMessage>,
    output_limit: u32,
    cancel: CancellationToken,
) -> Result<String, String> {
    let timeout = crate::services::compress::timeouts::compression_request_timeout();
    if provider == "ollama" {
        let compression =
            crate::services::agent_local::ollama_stream::collect_chat_with_timeout_and_limit(
                model,
                messages,
                timeout,
                Some(output_limit),
            );
        return tokio::select! {
            _ = cancel.cancelled() => Err("Annulé".to_string()),
            result = compression => result
                .map(|(content, _)| content)
                .map_err(|err| format!("Compression Ollama : {err}")),
        };
    }
    let purpose =
        crate::services::llm::request_purpose::RequestPurpose::for_session(session_id).await;
    let result = crate::services::llm::stream::collect_chat_silent_for_compression(
        provider,
        model,
        &messages,
        output_limit,
        purpose,
        cancel,
    )
    .await?;
    crate::services::provider_usage::record_for_session(
        provider,
        model,
        session_id,
        crate::services::provider_usage::UsageWorkload::Compression,
        result.usage.as_ref(),
    )
    .await;
    Ok(result.content)
}

async fn resolve_context_window(provider: &str, model: &str) -> u64 {
    let ctx = if provider == "ollama" {
        crate::services::compress::context_resolve::resolve_ollama(model).await
    } else {
        crate::services::compress::context_resolve::resolve_api(provider, model).await
    };
    ctx.configured
}

fn send_compression_done(on_event: &AgentEventEmitter) {
    send_compressing_done(on_event);
    let _ = on_event.send(StreamEvent::CompressionComplete {});
    let _ = on_event.send(StreamEvent::Done {
        eval_count: None,
        eval_duration_ns: 0,
        final_tps: 0.0,
        prompt_tokens: None,
        context_tokens: None,
    });
}

fn send_compressing_done(on_event: &AgentEventEmitter) {
    let _ = on_event.send(StreamEvent::Compressing {
        status: "done".to_string(),
    });
}
