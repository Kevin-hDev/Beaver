use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::ChatMessage;
use std::path::Path;
use tokio_util::sync::CancellationToken;

pub enum ToolCompressionProvider<'a> {
    #[allow(
        dead_code,
        reason = "journal commits tool results before compression can resume"
    )]
    Ollama { model: &'a str },
    Cloud {
        provider_id: &'a str,
        model: &'a str,
        fast_mode: crate::services::llm::fast_mode::FastModeRequest,
    },
}

pub struct ToolCompression<'a> {
    pub on_event: &'a AgentEventEmitter,
    pub provider: ToolCompressionProvider<'a>,
    pub session_id: &'a str,
    pub request_id: &'a str,
    pub configured_context: u64,
    pub provider_tools: &'a [serde_json::Value],
    pub chatbot: bool,
    pub plan_mode_active: bool,
    pub working_dir: &'a Path,
    pub cancel: CancellationToken,
}

impl ToolCompression<'_> {
    pub async fn try_run(&self, messages: &mut Vec<ChatMessage>) -> bool {
        let (provider_id, model, fast_mode) = match self.provider {
            ToolCompressionProvider::Ollama { model } => (
                "ollama",
                model,
                crate::services::llm::fast_mode::FastModeRequest::Unsupported,
            ),
            ToolCompressionProvider::Cloud {
                provider_id,
                model,
                fast_mode,
            } => (provider_id, model, fast_mode),
        };
        crate::services::compress::automatic_run::try_run(
            crate::services::compress::automatic_run::AutomaticCompressionRequest {
                on_event: self.on_event,
                provider_id,
                fast_mode,
                model,
                messages,
                session_id: self.session_id,
                request_id: self.request_id,
                configured_context: self.configured_context,
                provider_tools: self.provider_tools,
                chatbot: self.chatbot,
                plan_mode_active: self.plan_mode_active,
                working_dir: self.working_dir,
                cancel: self.cancel.clone(),
            },
        )
        .await
        .is_some()
    }
}
