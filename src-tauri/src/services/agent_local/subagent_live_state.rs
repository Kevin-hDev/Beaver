use super::types_session::{AgentSession, AgentSessionMeta};

pub async fn normalize_session(mut session: AgentSession) -> AgentSession {
    if let Some(run_id) = super::subagent_registry::get_run_id_for_child(&session.id).await {
        mark_running(
            &mut session.subagent_status,
            &mut session.subagent_run_id,
            run_id,
        );
    } else if queued_prompts_keep_running(&session) {
        session.subagent_status = Some(super::subagent_status::RUNNING.to_string());
    }
    session
}

pub async fn normalize_meta(mut meta: AgentSessionMeta) -> AgentSessionMeta {
    if let Some(run_id) = super::subagent_registry::get_run_id_for_child(&meta.id).await {
        mark_running(&mut meta.subagent_status, &mut meta.subagent_run_id, run_id);
    }
    meta
}

pub async fn has_pending_work(session: &AgentSession) -> bool {
    saved_pending_work(session)
        || super::subagent_registry::get_run_id_for_child(&session.id)
            .await
            .is_some()
}

pub fn saved_pending_work(session: &AgentSession) -> bool {
    session.subagent_status.as_deref() == Some(super::subagent_status::RUNNING)
        || queued_prompts_keep_running(session)
}

fn mark_running(status: &mut Option<String>, run_id_slot: &mut Option<String>, run_id: String) {
    *status = Some(super::subagent_status::RUNNING.to_string());
    *run_id_slot = Some(run_id);
}

fn queued_prompts_keep_running(session: &AgentSession) -> bool {
    !session.subagent_queued_prompts.is_empty()
        && !matches!(
            session.subagent_status.as_deref(),
            Some(super::subagent_status::FAILED)
                | Some(super::subagent_status::CANCELLED)
                | Some(super::subagent_status::INTERRUPTED)
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::agent_local::types_session::AgentSession;
    use chrono::Utc;

    #[test]
    fn queued_prompt_is_pending_even_if_saved_completed() {
        let mut child = child("child", "completed");
        child.subagent_queued_prompts.push("suite".into());

        assert!(saved_pending_work(&child));
    }

    #[test]
    fn queued_prompt_does_not_keep_failed_child_running() {
        let mut child = child("child", "failed");
        child.subagent_queued_prompts.push("suite".into());

        assert!(!saved_pending_work(&child));
    }

    #[tokio::test]
    async fn normalize_session_marks_queued_completed_child_running() {
        let mut child = child("child", "completed");
        child.subagent_run_id = None;
        child.subagent_queued_prompts.push("suite".into());

        let normalized = normalize_session(child).await;

        assert_eq!(
            normalized.subagent_status.as_deref(),
            Some(super::super::subagent_status::RUNNING)
        );
    }

    fn child(id: &str, status: &str) -> AgentSession {
        AgentSession {
            id: id.into(),
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
