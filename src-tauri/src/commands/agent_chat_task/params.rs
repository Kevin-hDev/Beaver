use crate::services::agent_local::stream_events::AgentEventEmitter;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone, Default)]
#[allow(
    dead_code,
    reason = "legacy internal producers cannot govern canonical chat"
)]
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
    pub conversation: Option<super::conversation::StreamConversation>,
    pub continuation_target:
        Option<crate::services::reasoning_continuity::contract::ContinuationTarget>,
    pub reasoning_profile: Option<crate::services::reasoning_profile::EffectiveReasoningProfile>,
    pub tools: Vec<serde_json::Value>,
    pub think: bool,
    pub provider: String,
    pub working_dir: std::path::PathBuf,
    pub outputs_dir: Option<std::path::PathBuf>,
    pub capability_hints: StreamCapabilityHints,
    pub reasoning_mode: Option<String>,
    pub permission_mode: StreamPermissionMode,
    pub permission_emitter: Option<AgentEventEmitter>,
    pub parent_message_inbox: Option<
        std::sync::Arc<crate::services::agent_local::parent_message_inbox::ParentMessageInbox>,
    >,
    pub subagent_profile:
        Option<crate::services::agent_local::subagent_tool_profile::SubagentToolProfile>,
    pub plan_mode: Option<bool>,
    /// Présent uniquement pour une fixture debug : le contexte reste propriétaire
    /// de son TempDir jusqu'à la fin exacte de cette exécution de boucle.
    #[cfg(debug_assertions)]
    pub fixture_run: Option<crate::services::reasoning_fixture_run::FixtureRunContext>,
    pub cancel: CancellationToken,
}

pub(crate) enum StreamPermissionMode {
    Bounded(Option<String>),
    FullAccess,
}
