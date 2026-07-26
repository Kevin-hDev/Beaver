use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::ChatMessage;
use std::path::Path;
use tokio_util::sync::CancellationToken;

pub enum ToolCompressionProvider<'a> {
    Ollama {
        model: &'a str,
    },
    Cloud {
        provider_id: &'a str,
        model: &'a str,
    },
}

pub struct ToolCompression<'a> {
    pub on_event: &'a AgentEventEmitter,
    pub provider: ToolCompressionProvider<'a>,
    pub session_id: &'a str,
    pub request_id: &'a str,
    pub native_context: u64,
    pub configured_context: u64,
    pub last_context_tokens: Option<u32>,
    pub working_dir: &'a Path,
    pub cancel: CancellationToken,
}

impl ToolCompression<'_> {
    pub async fn try_run(&self, messages: &mut Vec<ChatMessage>) -> bool {
        match self.provider {
            ToolCompressionProvider::Ollama { model } => {
                crate::services::agent_local::compress_hook::try_auto_compress(
                    self.on_event,
                    messages,
                    model,
                    self.session_id,
                    self.request_id,
                    self.native_context,
                    self.configured_context,
                    self.last_context_tokens,
                    self.working_dir,
                    self.cancel.clone(),
                )
                .await
                .is_some()
            }
            ToolCompressionProvider::Cloud { provider_id, model } => {
                crate::services::llm::compress_hook::try_auto_compress(
                    self.on_event,
                    provider_id,
                    model,
                    messages,
                    self.session_id,
                    self.request_id,
                    self.native_context,
                    self.configured_context,
                    self.last_context_tokens,
                    self.working_dir,
                    self.cancel.clone(),
                )
                .await
                .is_some()
            }
        }
    }
}
