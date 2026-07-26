use super::types_session::AgentSession;

#[derive(Default)]
pub(super) struct TodoHousekeeping {
    pub should_emit_empty_update: bool,
}

pub(super) fn apply_user_turn(
    session: &mut AgentSession,
    has_user_message: bool,
) -> TodoHousekeeping {
    if !has_user_message {
        return TodoHousekeeping::default();
    }

    if super::types_todo::all_completed(&session.todos) {
        session.todos.clear();
        session.active_todo_run_id = None;
        session.todo_neglect_count = 0;
        return TodoHousekeeping {
            should_emit_empty_update: true,
        };
    }

    TodoHousekeeping {
        should_emit_empty_update: super::tool_todo_neglect::record_user_turn(session),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::agent_local::tool_todo_parse::parse_todos;
    use crate::services::agent_local::tool_todo_state::apply_todos_to_session;
    use crate::services::agent_local::types_session::AgentSession;
    use chrono::Utc;
    use serde_json::json;

    #[test]
    fn clears_completed_visible_todo_on_next_user_turn() {
        let mut session = test_session();
        apply_todos_to_session(
            &mut session,
            parse_todos(&json!({
                "todos": [{"content": "Done", "status": "completed"}]
            }))
            .unwrap(),
        );
        session.todo_neglect_count = 3;

        let housekeeping = apply_user_turn(&mut session, true);

        assert!(housekeeping.should_emit_empty_update);
        assert!(session.todos.is_empty());
        assert!(session.active_todo_run_id.is_none());
        assert_eq!(session.todo_neglect_count, 0);
    }

    #[test]
    fn keeps_completed_visible_todo_without_user_turn() {
        let mut session = test_session();
        apply_todos_to_session(
            &mut session,
            parse_todos(&json!({
                "todos": [{"content": "Done", "status": "completed"}]
            }))
            .unwrap(),
        );

        let housekeeping = apply_user_turn(&mut session, false);

        assert!(!housekeeping.should_emit_empty_update);
        assert_eq!(session.todos.len(), 1);
        assert!(session.active_todo_run_id.is_some());
    }

    fn test_session() -> AgentSession {
        AgentSession {
            id: "abc-123".into(),
            name: "Test".into(),
            created_at: Utc::now(),
            updated_at: None,
            archived_at: None,
            model: "llama3".into(),
            provider: "ollama".into(),
            thinking_enabled: false,
            reasoning_mode: None,
            accumulated_tokens: 0,
            messages: vec![],
            todos: vec![],
            todo_neglect_count: 0,
            todo_runs: vec![],
            active_todo_run_id: None,
            stream_failures: vec![],
            diagnostic_runs: vec![],
            plan_mode_enabled: false,
            plan_runs: vec![],
            active_plan_id: None,
            plan_workflow_status: Default::default(),
            plan_approval_decision: None,
            is_heartbeat: false,
            is_gateway: false,
            gateway_channel_key: None,
            project_id: None,
            working_dir: String::new(),
            parent_session_id: None,
            subagent_type: None,
            subagent_worktree: None,
            subagent_prompt: None,
            subagent_status: None,
            subagent_run_id: None,
            subagent_description: None,
            subagent_color_key: None,
            subagent_summary: None,
            subagent_last_activity: None,
            subagent_queued_prompts: Vec::new(),
            subagent_hidden_reports: Vec::new(),
            clone_parent_session_id: None,
            clone_parent_message_id: None,
            clone_mode: None,
            clone_summary: None,
            clone_read_files: Vec::new(),
            clone_modified_files: Vec::new(),
            clone_root_session_id: None,
            git_branch: None,
        }
    }
}
