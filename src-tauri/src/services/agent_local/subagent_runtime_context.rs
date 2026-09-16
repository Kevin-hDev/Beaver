#[derive(Clone)]
pub struct SubagentRuntimeContext {
    pub permission_mode: String,
}

impl SubagentRuntimeContext {
    pub async fn from_parent(parent: &super::types_session::AgentSession) -> Self {
        let global = super::agent_settings::get_permission_mode().await;
        Self::from_parent_with_global(parent, &global).await
    }

    async fn from_parent_with_global(
        parent: &super::types_session::AgentSession,
        global: &str,
    ) -> Self {
        let parent_mode = super::session_permission_state::load(&parent.id)
            .await
            .map(|state| state.permission_mode.as_str().to_string())
            .unwrap_or_else(|_| "chat".to_string());
        Self {
            permission_mode: most_restrictive(&parent_mode, global).to_string(),
        }
    }
}

fn most_restrictive<'a>(left: &'a str, right: &'a str) -> &'a str {
    if !matches!(left, "chat" | "manual" | "auto")
        || !matches!(right, "chat" | "manual" | "auto")
    {
        return "chat";
    }
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
        assert_eq!(super::most_restrictive("invalid", "auto"), "chat");
        assert_eq!(super::most_restrictive("auto", "invalid"), "chat");
    }

    #[tokio::test]
    async fn child_reads_the_persisted_parent_mode_before_applying_the_global_cap() {
        let parent = super::super::session_store::create_full(
            "Parent permissions",
            "model",
            "provider",
            false,
            None,
        )
        .await
        .unwrap();
        super::super::session_permission_state::set_mode(
            &parent.id,
            super::super::session_permission_state::PermissionMode::Manual,
        )
        .await
        .unwrap();

        let context = super::SubagentRuntimeContext::from_parent_with_global(&parent, "auto").await;

        assert_eq!(context.permission_mode, "manual");
        super::super::session_store::delete_one(&parent.id)
            .await
            .unwrap();
    }
}
