use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::ChatMessage;
use std::path::Path;
use tokio_util::sync::CancellationToken;

pub struct AutomaticCompressionRequest<'a> {
    pub on_event: &'a AgentEventEmitter,
    pub provider_id: &'a str,
    pub fast_mode: crate::services::llm::fast_mode::FastModeRequest,
    pub model: &'a str,
    pub messages: &'a mut Vec<ChatMessage>,
    pub session_id: &'a str,
    pub request_id: &'a str,
    pub configured_context: u64,
    pub provider_tools: &'a [serde_json::Value],
    pub chatbot: bool,
    pub plan_mode_active: bool,
    pub working_dir: &'a Path,
    pub cancel: CancellationToken,
}

pub async fn try_run(request: AutomaticCompressionRequest<'_>) -> Option<u32> {
    let prepared_count = super::prepared_request::count(
        request.provider_id,
        request.model,
        request.messages,
        request.provider_tools,
    );
    match super::orchestrator::run_compression(super::orchestrator::CompressionRunRequest {
        on_event: request.on_event,
        session_id: request.session_id,
        request_id: request.request_id,
        trigger: super::profile_types::CompressionTrigger::Automatic,
        runtime_messages: request.messages,
        provider_id: request.provider_id,
        fast_mode: request.fast_mode,
        context_window: request.configured_context,
        prepared_count,
        provider_tools: request.provider_tools,
        chatbot: request.chatbot,
        plan_mode_active: request.plan_mode_active,
        working_dir: request.working_dir,
        cancel: request.cancel,
    })
    .await
    {
        Ok(Some(report)) => Some(report.after_tokens),
        Ok(None) | Err(_) => None,
    }
}
