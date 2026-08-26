use super::super::agent_chat_task::StreamCapabilityHints;
use crate::models::agent_turn_contract::TurnStart;

pub(crate) struct ChatStreamRequest {
    pub session_id: String,
    pub model: String,
    pub turn: Option<TurnStart>,
    pub tools: Vec<serde_json::Value>,
    pub think: bool,
    pub provider: String,
    pub working_dir: Option<String>,
    pub capability_hints: StreamCapabilityHints,
    pub reasoning_mode: Option<String>,
    pub permission_mode: Option<String>,
    pub plan_mode: Option<bool>,
    #[cfg(debug_assertions)]
    pub fixture_run: Option<crate::services::reasoning_fixture_run::FixtureRunContext>,
}

impl ChatStreamRequest {
    pub(crate) fn from_input(
        input: crate::models::agent_turn_contract::ChatStreamRequestInput,
    ) -> Self {
        Self {
            session_id: input.session_id,
            model: input.model,
            turn: Some(input.turn),
            tools: Vec::new(),
            think: false,
            provider: input.provider,
            working_dir: input.working_dir,
            capability_hints: StreamCapabilityHints::default(),
            reasoning_mode: None,
            permission_mode: input.permission_mode,
            plan_mode: input.plan_mode,
            #[cfg(debug_assertions)]
            fixture_run: None,
        }
    }
}

pub(super) fn generic_error() -> String {
    "conversation_admission_failed".to_string()
}
