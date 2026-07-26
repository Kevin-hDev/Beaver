use super::agent_loop_support;
use super::compress_hook;
use super::stream_events::AgentEventEmitter;
use super::tool_executor_compression::{ToolCompression, ToolCompressionProvider};
use super::types_ollama::{ChatMessage, StreamResult};
use crate::services::compress::token_estimate;
use crate::services::token_counting;
use std::path::Path;
use tokio_util::sync::CancellationToken;

pub(super) struct LoopCompression<'a> {
    pub on_event: &'a AgentEventEmitter,
    pub model: &'a str,
    pub session_id: &'a str,
    pub request_id: &'a str,
    pub native_context: u64,
    pub configured_context: u64,
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
        last_prompt: Option<u32>,
        last_eval: Option<u32>,
        cancel: CancellationToken,
    ) -> Option<u32> {
        self.try_run_context(
            messages,
            token_counting::sum_real_counts(last_prompt, last_eval),
            cancel,
        )
        .await
    }

    async fn try_run_context(
        &self,
        messages: &mut Vec<ChatMessage>,
        last_context_tokens: Option<u32>,
        cancel: CancellationToken,
    ) -> Option<u32> {
        compress_hook::try_auto_compress(
            self.on_event,
            messages,
            self.model,
            self.session_id,
            self.request_id,
            self.native_context,
            self.configured_context,
            last_context_tokens,
            self.working_dir,
            cancel,
        )
        .await
    }

    pub async fn handle_interrupted(
        &self,
        messages: &mut Vec<ChatMessage>,
        result: &StreamResult,
        counts: LastCounts<'_>,
        cancel: CancellationToken,
    ) -> Result<(), String> {
        messages.push(agent_loop_support::build_assistant_message(result));
        let context = token_estimate::estimate_tokens(messages)
            .saturating_add(result.content_chunks.len())
            .min(u32::MAX as usize) as u32;
        if self
            .try_run_context(messages, Some(context), cancel)
            .await
            .is_none()
        {
            return Err("Compression impossible après interruption du stream".to_string());
        }
        Self::reset_counts(counts.prompt, counts.eval);
        Ok(())
    }

    pub async fn try_run_and_reset(
        &self,
        messages: &mut Vec<ChatMessage>,
        last_prompt: &mut Option<u32>,
        last_eval: &mut Option<u32>,
        cancel: CancellationToken,
    ) -> bool {
        let compressed = self
            .try_run(messages, *last_prompt, *last_eval, cancel)
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
        compressed_during_tools: bool,
        last_prompt: &mut Option<u32>,
        last_eval: &mut Option<u32>,
        cancel: CancellationToken,
    ) -> bool {
        if compressed_during_tools {
            Self::reset_counts(last_prompt, last_eval);
            return true;
        }
        self.try_run_and_reset(messages, last_prompt, last_eval, cancel)
            .await
    }

    pub fn reset_counts(last_prompt: &mut Option<u32>, last_eval: &mut Option<u32>) {
        *last_prompt = None;
        *last_eval = None;
    }

    pub fn tool_compression<'a>(
        &'a self,
        last_context_tokens: Option<u32>,
        cancel: CancellationToken,
    ) -> ToolCompression<'a> {
        ToolCompression {
            on_event: self.on_event,
            provider: ToolCompressionProvider::Ollama { model: self.model },
            session_id: self.session_id,
            request_id: self.request_id,
            native_context: self.native_context,
            configured_context: self.configured_context,
            last_context_tokens,
            working_dir: self.working_dir,
            cancel,
        }
    }
}
