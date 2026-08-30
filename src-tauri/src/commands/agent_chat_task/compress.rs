#![expect(
    clippy::too_many_arguments,
    reason = "orchestration boundary keeps related runtime context explicit"
)]
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::ChatMessage;
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
    provider_tools: &[serde_json::Value],
    chatbot: bool,
    plan_mode_active: bool,
    working_dir: &Path,
    cancel: CancellationToken,
) -> Result<(), String> {
    let fast_mode =
        crate::services::llm::fast_mode::for_session(session_id, provider, model).await?;
    let mut runtime = messages.to_vec();
    let context = crate::services::compress::context_resolve::resolve(provider, model)
        .await
        .configured;
    match crate::services::compress::orchestrator::run_compression(
        crate::services::compress::orchestrator::CompressionRunRequest {
            on_event,
            session_id,
            request_id,
            trigger: crate::services::compress::profile_types::CompressionTrigger::Explicit,
            runtime_messages: &mut runtime,
            provider_id: provider,
            fast_mode,
            context_window: context,
            last_context_tokens: None,
            provider_tools,
            chatbot,
            plan_mode_active,
            working_dir,
            cancel,
        },
    )
    .await
    {
        Ok(Some(_)) => Ok(()),
        Ok(None) => Err("Compression indisponible.".to_string()),
        Err(error) => Err(error.public_message().to_string()),
    }
}
