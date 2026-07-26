#[derive(Clone)]
pub struct SubagentRuntimeContext {
    pub think: bool,
    pub reasoning_mode: Option<String>,
    pub permission_mode: String,
}

impl SubagentRuntimeContext {
    pub async fn from_parent(parent: &super::types_session::AgentSession) -> Self {
        Self {
            think: parent.thinking_enabled,
            reasoning_mode: parent.reasoning_mode.clone(),
            permission_mode: super::agent_settings::get_permission_mode().await,
        }
    }
}
