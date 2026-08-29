#![expect(
    clippy::too_many_arguments,
    reason = "orchestration boundary keeps related runtime context explicit"
)]
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

    let fast_mode =
        crate::services::llm::fast_mode::for_session(session_id, provider, model).await?;
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
    let context = crate::services::compress::context_resolve::resolve(provider, model)
        .await
        .configured;
    let (summary_instruction, output_limit) =
        crate::services::compress::summary_budget::summary_instruction_for_input(
            context,
            input_tokens,
        );
    ::log::info!(
        "[compress] manual start session={session_id} provider={provider} input_tokens={input_tokens} output_limit={output_limit}"
    );

    let compress_msgs = engine::build_compression_request_content(
        &msgs_without_command,
        summary_instruction.as_deref(),
    );
    let summary_raw = match collect_summary(
        provider,
        fast_mode,
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
            ::log::error!("[compress] manual failed session={session_id}: {err}");
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

    send_compression_complete(on_event);
    ::log::info!("[compress] manual done session={session_id} context_tokens={current_tokens}");
    Ok(())
}

async fn collect_summary(
    provider: &str,
    fast_mode: crate::services::llm::fast_mode::FastModeRequest,
    model: &str,
    session_id: &str,
    messages: Vec<ChatMessage>,
    output_limit: u32,
    cancel: CancellationToken,
) -> Result<String, String> {
    let purpose =
        crate::services::llm::request_purpose::RequestPurpose::for_session(session_id).await;
    let result = crate::services::llm::stream::collect_chat_silent_for_compression(
        provider,
        fast_mode,
        model,
        &messages,
        output_limit,
        purpose,
        session_id,
        None,
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

fn send_compression_complete(on_event: &AgentEventEmitter) {
    send_compressing_done(on_event);
    let _ = on_event.send(StreamEvent::CompressionComplete {});
}

fn send_compressing_done(on_event: &AgentEventEmitter) {
    let _ = on_event.send(StreamEvent::Compressing {
        status: "done".to_string(),
    });
}
