#[derive(Clone)]
pub struct SubagentRuntimeContext {
    pub permission_mode: String,
}

impl SubagentRuntimeContext {
    pub async fn from_parent(_parent: &super::types_session::AgentSession) -> Self {
        Self {
            permission_mode: super::agent_settings::get_permission_mode().await,
        }
    }
}
