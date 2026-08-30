#![expect(
    clippy::too_many_arguments,
    reason = "orchestration boundary keeps related runtime context explicit"
)]
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::ChatMessage;
use std::path::Path;
use tokio_util::sync::CancellationToken;

pub async fn try_auto_compress(
    on_event: &AgentEventEmitter,
    messages: &mut Vec<ChatMessage>,
    _model: &str,
    session_id: &str,
    request_id: &str,
    native_context: u64,
    configured_context: u64,
    last_context_tokens: Option<u32>,
    provider_tools: &[serde_json::Value],
    chatbot: bool,
    plan_mode_active: bool,
    working_dir: &Path,
    cancel: CancellationToken,
) -> Option<u32> {
    let _ = native_context;
    match crate::services::compress::orchestrator::run_compression(
        crate::services::compress::orchestrator::CompressionRunRequest {
            on_event,
            session_id,
            request_id,
            trigger: crate::services::compress::profile_types::CompressionTrigger::Automatic,
            runtime_messages: messages,
            provider_id: "ollama",
            fast_mode: crate::services::llm::fast_mode::FastModeRequest::Unsupported,
            context_window: configured_context,
            last_context_tokens,
            provider_tools,
            chatbot,
            plan_mode_active,
            working_dir,
            cancel,
        },
    )
    .await
    {
        Ok(Some(report)) => Some(report.after_tokens),
        Ok(None) | Err(_) => None,
    }
}

#[cfg(test)]
#[path = "compress_hook_tests.rs"]
mod tests;
