#[cfg(test)]
mod tests {
    use crate::services::agent_local::subagent_working_dir::create_coder_worktree_for_test;
    use crate::services::agent_local::types_session::AgentSession;
    use chrono::Utc;
    use uuid::Uuid;

    fn temp_dir() -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!("cl-go-subagent-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&path).expect("create test dir");
        path
    }

    #[tokio::test]
    async fn test_coder_worktree_creation_failure_is_blocking() {
        let project = temp_dir();
        let child_id = Uuid::new_v4().to_string();
        let run_id = Uuid::new_v4().to_string();
        let execution_id = Uuid::new_v4().to_string();
        let result = create_coder_worktree_for_test(
            &project,
            &child_id,
            &run_id,
            &execution_id,
        )
        .await;

        assert!(
            result.is_err(),
            "un sous-agent coder ne doit pas retomber dans le dossier principal si le worktree échoue"
        );

        let _ = std::fs::remove_dir_all(project);
    }

    #[test]
    fn queued_followup_keeps_session_running() {
        let status = super::super::subagent_task::effective_session_status("completed", true);

        assert_eq!(status, "running");
    }

    #[test]
    fn final_run_keeps_completed_status() {
        let status = super::super::subagent_task::effective_session_status("completed", false);

        assert_eq!(status, "completed");
    }

    #[test]
    fn running_activity_label_is_not_completed() {
        assert_eq!(
            super::super::subagent_task::final_activity_label("running"),
            "En cours"
        );
    }

    #[test]
    fn finalized_state_keeps_queued_child_running() {
        let mut child = child_session("completed");
        child.subagent_queued_prompts.push("suite".into());

        let finalized = super::super::subagent_task::apply_finalized_subagent_state(
            &mut child,
            "completed",
            "rapport",
        );

        assert!(finalized.queued_followup);
        assert_eq!(finalized.session_status, "running");
        assert_eq!(child.subagent_status.as_deref(), Some("running"));
        assert_eq!(child.subagent_queued_prompts, vec!["suite"]);
    }

    #[test]
    fn failed_or_interrupted_state_preserves_queue_for_explicit_redeployment() {
        for status in ["failed", "interrupted"] {
            let mut child = child_session("running");
            child.subagent_queued_prompts.push("suite".into());

            let finalized = super::super::subagent_task::apply_finalized_subagent_state(
                &mut child, status, "rapport",
            );

            assert!(!finalized.queued_followup);
            assert_eq!(finalized.session_status, status);
            assert_eq!(child.subagent_queued_prompts, vec!["suite"]);
            assert_eq!(child.subagent_summary.as_deref(), Some("rapport"));
        }
    }

    #[test]
    fn voluntary_cancellation_discards_queued_work() {
        let mut child = child_session("running");
        child.subagent_queued_prompts.push("suite".into());

        super::super::subagent_task::apply_finalized_subagent_state(
            &mut child,
            "cancelled",
            "annulé",
        );

        assert!(child.subagent_queued_prompts.is_empty());
    }

    fn child_session(status: &str) -> AgentSession {
        AgentSession {
            id: Uuid::new_v4().to_string(),
            name: "Geminitor".into(),
            created_at: Utc::now(),
            updated_at: Some(Utc::now()),
            archived_at: None,
            model: "llama3".into(),
            provider: "ollama".into(),
            thinking_enabled: false,
            reasoning_mode: None,
            accumulated_tokens: 0,
            messages: Vec::new(),
            todos: Vec::new(),
            todo_neglect_count: 0,
            todo_runs: Vec::new(),
            active_todo_run_id: None,
            stream_failures: Vec::new(),
            diagnostic_runs: Vec::new(),
            plan_mode_enabled: false,
            plan_runs: Vec::new(),
            active_plan_id: None,
            plan_workflow_status: Default::default(),
            plan_approval_decision: None,
            is_heartbeat: false,
            is_gateway: false,
            gateway_channel_key: None,
            project_id: None,
            working_dir: String::new(),
            parent_session_id: Some("parent".into()),
            subagent_type: Some("explorer".into()),
            subagent_worktree: None,
            subagent_prompt: None,
            subagent_status: Some(status.into()),
            subagent_run_id: Some("run-1".into()),
            subagent_description: Some("Analyse".into()),
            subagent_color_key: Some("geminitor".into()),
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
