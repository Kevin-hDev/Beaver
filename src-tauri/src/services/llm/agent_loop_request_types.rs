use crate::services::agent_local::{
    context_usage_buckets::ContextUsageSeed, stream_events::AgentEventEmitter,
    subagent_orchestration::ParentSubagentOrchestrator, types_ollama::ChatMessage,
};
use tokio_util::sync::CancellationToken;

use crate::services::agent_local::generation_metrics::GenerationAggregate;
use crate::services::agent_local::types_ollama::StreamResult;

pub(super) struct ApiRequestOutput {
    pub result: StreamResult,
    pub plan_active: bool,
    pub interrupted: bool,
    pub input_tokens: u32,
    pub generation: GenerationAggregate,
}

pub(super) struct ApiRequestParams<'a> {
    pub on_event: &'a AgentEventEmitter,
    pub provider_id: &'a str,
    pub fast_mode: super::fast_mode::FastModeRequest,
    pub model: &'a str,
    pub messages: &'a mut Vec<ChatMessage>,
    pub tools: &'a [serde_json::Value],
    pub think: bool,
    pub reasoning_mode: Option<&'a str>,
    pub session_id: &'a str,
    pub request_id: &'a str,
    pub cancel: CancellationToken,
    pub configured_context: u64,
    pub plan_mode_active: bool,
    pub turn: usize,
    pub subagents: &'a mut ParentSubagentOrchestrator,
    pub context_usage_seed: ContextUsageSeed,
    pub tool_result_previews:
        &'a crate::services::agent_local::tool_artifact_preview::ToolResultPreviewBatch,
    pub continuation_target:
        Option<crate::services::reasoning_continuity::contract::ContinuationTarget>,
}
