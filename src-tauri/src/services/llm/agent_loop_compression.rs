use super::agent_loop_message;
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::{ChatMessage, StreamResult};
use std::path::Path;
use tokio_util::sync::CancellationToken;

pub(super) struct LoopCompression<'a> {
    pub on_event: &'a AgentEventEmitter,
    pub provider_id: &'a str,
    pub fast_mode: super::fast_mode::FastModeRequest,
    pub model: &'a str,
    pub session_id: &'a str,
    pub request_id: &'a str,
    pub configured_context: u64,
    pub chatbot: bool,
    pub plan_mode_active: bool,
    pub working_dir: &'a Path,
}

pub(super) struct LastCounts<'a> {
    pub prompt: &'a mut Option<u32>,
    pub eval: &'a mut Option<u32>,
}

impl<'a> LastCounts<'a> {
    pub fn new(prompt: &'a mut Option<u32>, eval: &'a mut Option<u32>) -> Self {
        Self { prompt, eval }
    }
}

impl LoopCompression<'_> {
    pub async fn try_run(
        &self,
        messages: &mut Vec<ChatMessage>,
        provider_tools: &[serde_json::Value],
        cancel: CancellationToken,
    ) -> Option<u32> {
        crate::services::compress::automatic_run::try_run(
            crate::services::compress::automatic_run::AutomaticCompressionRequest {
                on_event: self.on_event,
                provider_id: self.provider_id,
                fast_mode: self.fast_mode,
                model: self.model,
                messages,
                session_id: self.session_id,
                request_id: self.request_id,
                configured_context: self.configured_context,
                provider_tools,
                chatbot: self.chatbot,
                plan_mode_active: self.plan_mode_active,
                working_dir: self.working_dir,
                cancel,
            },
        )
        .await
    }

    pub async fn handle_interrupted(
        &self,
        messages: &mut Vec<ChatMessage>,
        provider_tools: &[serde_json::Value],
        result: &StreamResult,
        counts: LastCounts<'_>,
        cancel: CancellationToken,
    ) -> Result<(), String> {
        messages.push(agent_loop_message::build_assistant_message(result));
        if self
            .try_run(messages, provider_tools, cancel)
            .await
            .is_none()
        {
            return Err("compression_failed".to_string());
        }
        Self::reset_counts(counts.prompt, counts.eval);
        Ok(())
    }

    pub async fn try_run_and_reset(
        &self,
        messages: &mut Vec<ChatMessage>,
        provider_tools: &[serde_json::Value],
        last_prompt: &mut Option<u32>,
        last_eval: &mut Option<u32>,
        cancel: CancellationToken,
    ) -> bool {
        let compressed = self
            .try_run(messages, provider_tools, cancel)
            .await
            .is_some();
        if compressed {
            Self::reset_counts(last_prompt, last_eval);
        }
        compressed
    }

    pub async fn after_tools(
        &self,
        messages: &mut Vec<ChatMessage>,
        provider_tools: &[serde_json::Value],
        compressed_during_tools: bool,
        last_prompt: &mut Option<u32>,
        last_eval: &mut Option<u32>,
        cancel: CancellationToken,
    ) -> bool {
        if compressed_during_tools {
            Self::reset_counts(last_prompt, last_eval);
            return true;
        }
        self.try_run_and_reset(messages, provider_tools, last_prompt, last_eval, cancel)
            .await
    }

    pub async fn finish_tools(
        &self,
        messages: &mut Vec<ChatMessage>,
        provider_tools: &[serde_json::Value],
        compressed_during_tools: bool,
        counts: LastCounts<'_>,
        cancel: CancellationToken,
    ) {
        let compressed = self
            .after_tools(
                messages,
                provider_tools,
                compressed_during_tools,
                counts.prompt,
                counts.eval,
                cancel,
            )
            .await;
        crate::services::agent_local::agent_loop_finish::emit_turn_end(self.on_event, compressed);
    }

    pub fn reset_counts(last_prompt: &mut Option<u32>, last_eval: &mut Option<u32>) {
        *last_prompt = None;
        *last_eval = None;
    }
}

#[cfg(test)]
#[path = "agent_loop_compression_fast_mode_tests.rs"]
mod fast_mode_tests;
