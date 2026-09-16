#[derive(Clone)]
pub struct SubagentRuntimeContext {
    pub permission_mode: String,
}

impl SubagentRuntimeContext {
    pub async fn from_parent(parent: &super::types_session::AgentSession) -> Self {
        let parent_mode = super::session_permission_state::load(&parent.id)
            .await
            .map(|state| state.permission_mode.as_str().to_string())
            .unwrap_or_else(|_| "chat".to_string());
        let global = super::agent_settings::get_permission_mode().await;
        Self {
            permission_mode: most_restrictive(&parent_mode, &global).to_string(),
        }
    }
}

fn most_restrictive<'a>(left: &'a str, right: &'a str) -> &'a str {
    let rank = |value| match value {
        "chat" => 0,
        "manual" => 1,
        "auto" => 2,
        _ => 0,
    };
    if rank(left) <= rank(right) { left } else { right }
}

#[cfg(test)]
mod tests {
    #[test]
    fn child_inherits_effective_parent_permissions() {
        assert_eq!(super::most_restrictive("manual", "auto"), "manual");
        assert_eq!(super::most_restrictive("auto", "manual"), "manual");
        assert_eq!(super::most_restrictive("chat", "auto"), "chat");
        assert_eq!(super::most_restrictive("invalid", "auto"), "invalid");
    }
}
