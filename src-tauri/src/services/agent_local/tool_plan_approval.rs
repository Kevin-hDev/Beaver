use super::stream_events::AgentEventEmitter;
use super::tool_plan_approval_request::{
    APPROVAL_ID_CONTINUE, APPROVAL_ID_IMPLEMENT, APPROVAL_ID_QUIT,
};
use super::types_interactive::AgentInteractiveAnswer;
use super::types_plan::{AgentPlanApprovalDecision, AgentPlanStatus, AgentPlanWorkflowStatus};
use super::types_session::AgentSession;
use super::types_stream::StreamEvent;

pub async fn record_answers(
    session_id: &str,
    answers: &[AgentInteractiveAnswer],
    on_event: &AgentEventEmitter,
) -> Result<Option<&'static str>, String> {
    let lock = super::session_store::lock_session(session_id).await;
    let _guard = lock.lock().await;
    let mut session = super::session_store::get(session_id).await?;
    if !session.plan_mode_enabled
        || session.plan_workflow_status != AgentPlanWorkflowStatus::AwaitingApproval
    {
        return Ok(None);
    }
    match classify_answers(answers) {
        Some(AgentPlanApprovalDecision::Implement) => {
            session.plan_workflow_status = AgentPlanWorkflowStatus::Approved;
            session.plan_approval_decision = Some(AgentPlanApprovalDecision::Implement);
            super::session_store::save(&session).await?;
            Ok(Some("implement"))
        }
        Some(AgentPlanApprovalDecision::ContinuePlanning) => {
            mark_active_run(&mut session, AgentPlanStatus::Rejected);
            session.plan_workflow_status = AgentPlanWorkflowStatus::CollectingQuestions;
            session.plan_approval_decision = Some(AgentPlanApprovalDecision::ContinuePlanning);
            session.active_plan_id = None;
            super::session_store::save(&session).await?;
            let _ = on_event.send(StreamEvent::PlanPreviewUpdated { plan: None });
            Ok(Some("continue_planning"))
        }
        Some(AgentPlanApprovalDecision::QuitPlan) => {
            session.plan_workflow_status = AgentPlanWorkflowStatus::Rejected;
            session.plan_approval_decision = Some(AgentPlanApprovalDecision::QuitPlan);
            super::session_store::save(&session).await?;
            Ok(Some("quit_plan"))
        }
        None => Ok(Some("approval_unrecognized")),
    }
}

pub fn validate_exit(session: &AgentSession, status: AgentPlanStatus) -> Result<(), String> {
    match status {
        AgentPlanStatus::Approved => {
            if session.plan_workflow_status == AgentPlanWorkflowStatus::Approved
                && session.plan_approval_decision == Some(AgentPlanApprovalDecision::Implement)
            {
                Ok(())
            } else {
                Err("User approval is required before exiting Plan Mode.".to_string())
            }
        }
        AgentPlanStatus::Rejected => {
            if session.plan_approval_decision == Some(AgentPlanApprovalDecision::QuitPlan) {
                Ok(())
            } else {
                Err("User choice is required before exiting Plan Mode.".to_string())
            }
        }
        AgentPlanStatus::Cancelled => {
            Err("User choice is required before cancelling Plan Mode.".to_string())
        }
        _ => Err("Invalid plan status.".to_string()),
    }
}

pub(crate) fn classify_answers(
    answers: &[AgentInteractiveAnswer],
) -> Option<AgentPlanApprovalDecision> {
    let ids = answers
        .iter()
        .flat_map(|answer| answer.selected_ids.iter())
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join(" ");
    match ids.as_str() {
        APPROVAL_ID_IMPLEMENT => Some(AgentPlanApprovalDecision::Implement),
        APPROVAL_ID_CONTINUE => Some(AgentPlanApprovalDecision::ContinuePlanning),
        APPROVAL_ID_QUIT => Some(AgentPlanApprovalDecision::QuitPlan),
        _ => None,
    }
}

fn mark_active_run(session: &mut AgentSession, status: AgentPlanStatus) {
    if let Some(active_id) = session.active_plan_id.clone() {
        if let Some(run) = session.plan_runs.iter_mut().find(|run| run.id == active_id) {
            run.status = status;
            run.updated_at = chrono::Utc::now();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::agent_local::types_session::AgentMessage;
    use chrono::Utc;

    fn answer(label: &str) -> AgentInteractiveAnswer {
        AgentInteractiveAnswer {
            question_index: 0,
            selected_ids: vec![],
            selected_labels: vec![label.to_string()],
            custom_answer: None,
        }
    }

    fn answer_id(id: &str) -> AgentInteractiveAnswer {
        AgentInteractiveAnswer {
            question_index: 0,
            selected_ids: vec![id.to_string()],
            selected_labels: vec!["label".to_string()],
            custom_answer: None,
        }
    }

    fn session(
        workflow: AgentPlanWorkflowStatus,
        decision: Option<AgentPlanApprovalDecision>,
    ) -> AgentSession {
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
            messages: Vec::<AgentMessage>::new(),
            todos: vec![],
            todo_neglect_count: 0,
            todo_runs: vec![],
            active_todo_run_id: None,
            stream_failures: vec![],
            diagnostic_runs: vec![],
            plan_mode_enabled: true,
            plan_runs: vec![],
            active_plan_id: None,
            plan_workflow_status: workflow,
            plan_approval_decision: decision,
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

    #[test]
    fn classifies_plan_approval_choices_by_stable_id() {
        assert_eq!(
            classify_answers(&[answer_id(APPROVAL_ID_IMPLEMENT)]),
            Some(AgentPlanApprovalDecision::Implement)
        );
        assert_eq!(
            classify_answers(&[answer_id(APPROVAL_ID_CONTINUE)]),
            Some(AgentPlanApprovalDecision::ContinuePlanning)
        );
        assert_eq!(
            classify_answers(&[answer_id(APPROVAL_ID_QUIT)]),
            Some(AgentPlanApprovalDecision::QuitPlan)
        );
    }

    #[test]
    fn labels_do_not_approve_plan_without_id() {
        assert_eq!(
            classify_answers(&[answer("Mettre en oeuvre le plan")]),
            None
        );
    }

    #[test]
    fn approved_exit_requires_user_approval() {
        let blocked = session(AgentPlanWorkflowStatus::AwaitingApproval, None);
        assert!(validate_exit(&blocked, AgentPlanStatus::Approved).is_err());

        let allowed = session(
            AgentPlanWorkflowStatus::Approved,
            Some(AgentPlanApprovalDecision::Implement),
        );
        assert!(validate_exit(&allowed, AgentPlanStatus::Approved).is_ok());
    }

    #[test]
    fn rejected_exit_requires_quit_plan_decision() {
        let continue_planning = session(
            AgentPlanWorkflowStatus::CollectingQuestions,
            Some(AgentPlanApprovalDecision::ContinuePlanning),
        );
        assert!(validate_exit(&continue_planning, AgentPlanStatus::Rejected).is_err());

        let quit = session(
            AgentPlanWorkflowStatus::Rejected,
            Some(AgentPlanApprovalDecision::QuitPlan),
        );
        assert!(validate_exit(&quit, AgentPlanStatus::Rejected).is_ok());
    }

    #[test]
    fn cancelled_exit_requires_user_path() {
        let session = session(AgentPlanWorkflowStatus::AwaitingApproval, None);
        assert!(validate_exit(&session, AgentPlanStatus::Cancelled).is_err());
    }
}
