use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::ChatMessage;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone, Default)]
pub(crate) struct StreamCapabilityHints {
    pub supports_tools: Option<bool>,
    pub supports_thinking: Option<bool>,
    pub supports_vision: Option<bool>,
}

pub(crate) struct StreamTaskParams {
    pub on_event: AgentEventEmitter,
    pub session_id: String,
    pub request_id: String,
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub tools: Vec<serde_json::Value>,
    pub think: bool,
    pub provider: String,
    pub working_dir: Option<String>,
    pub capability_hints: StreamCapabilityHints,
    pub reasoning_mode: Option<String>,
    pub permission_mode_override: Option<String>,
    pub permission_emitter: Option<AgentEventEmitter>,
    pub parent_message_inbox: Option<
        std::sync::Arc<crate::services::agent_local::parent_message_inbox::ParentMessageInbox>,
    >,
    pub subagent_profile:
        Option<crate::services::agent_local::subagent_tool_profile::SubagentToolProfile>,
    pub plan_mode: Option<bool>,
    pub cancel: CancellationToken,
}
