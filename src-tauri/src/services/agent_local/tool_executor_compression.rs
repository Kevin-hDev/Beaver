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
        fast_mode: crate::services::llm::fast_mode::FastModeRequest,
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
            ToolCompressionProvider::Cloud {
                provider_id,
                model,
                fast_mode,
            } => {
                crate::services::llm::compress_hook::try_auto_compress(
                    self.on_event,
                    provider_id,
                    fast_mode,
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

#[cfg(test)]
mod tests {
    use super::ToolCompressionProvider;
    use crate::services::llm::fast_mode::FastModeRequest;

    #[test]
    fn tool_executor_compression_carries_the_generation_capture() {
        let provider = ToolCompressionProvider::Cloud {
            provider_id: "openai",
            model: "gpt-5.6-luna",
            fast_mode: FastModeRequest::Fast,
        };

        let ToolCompressionProvider::Cloud { fast_mode, .. } = provider else {
            panic!("cloud compression expected");
        };
        assert_eq!(fast_mode, FastModeRequest::Fast);
    }
}
