use super::*;
use crate::services::agent_local::types_interactive::AgentInteractiveAnswer;
use crate::services::agent_local::types_plan::{AgentPlanRun, AgentPlanWorkflowStatus};
use crate::services::agent_local::types_session::AgentMessage;
use chrono::Utc;

fn answer(id: &str, custom_answer: Option<&str>) -> InteractiveChoiceResponse {
    InteractiveChoiceResponse::Answered(vec![AgentInteractiveAnswer {
        question_index: 0,
        selected_ids: vec![id.to_string()],
        selected_labels: vec![],
        custom_answer: custom_answer.map(str::to_string),
    }])
}

#[test]
fn classifies_each_plan_approval_outcome() {
    assert_eq!(
        classify_response(answer(APPROVAL_ID_IMPLEMENT, None)).unwrap(),
        PlanApprovalOutcome::Implement
    );
    assert_eq!(
        classify_response(answer("other", Some("Changer la cible"))).unwrap(),
        PlanApprovalOutcome::Adjustments("Changer la cible".into())
    );
    assert_eq!(
        classify_response(InteractiveChoiceResponse::Dismissed).unwrap(),
        PlanApprovalOutcome::Dismissed
    );
}

#[test]
fn rejects_labels_and_empty_adjustments() {
    let labels_only = InteractiveChoiceResponse::Answered(vec![AgentInteractiveAnswer {
        question_index: 0,
        selected_ids: vec![],
        selected_labels: vec!["Mettre en oeuvre ce plan".into()],
        custom_answer: None,
    }]);
    assert!(classify_response(labels_only).is_err());
    assert!(classify_response(answer("other", Some("  "))).is_err());
}

#[test]
fn implementation_closes_plan_mode_and_approves_run() {
    let mut session = session();
    apply_outcome(&mut session, &PlanApprovalOutcome::Implement);

    assert!(!session.plan_mode_enabled);
    assert_eq!(
        session.plan_workflow_status,
        AgentPlanWorkflowStatus::Approved
    );
    assert_eq!(session.plan_runs[0].status, AgentPlanStatus::Approved);
    assert!(session.active_plan_id.is_none());
}

#[test]
fn adjustments_and_dismissal_keep_plan_mode_without_preview() {
    for (outcome, status) in [
        (
            PlanApprovalOutcome::Adjustments("Changer la cible".into()),
            AgentPlanStatus::Rejected,
        ),
        (PlanApprovalOutcome::Dismissed, AgentPlanStatus::Cancelled),
    ] {
        let mut session = session();
        apply_outcome(&mut session, &outcome);

        assert!(session.plan_mode_enabled);
        assert_eq!(
            session.plan_workflow_status,
            AgentPlanWorkflowStatus::CollectingQuestions
        );
        assert_eq!(session.plan_runs[0].status, status);
        assert!(session.active_plan_id.is_none());
    }
}

fn session() -> AgentSession {
    let now = Utc::now();
    AgentSession {
        schema_version:
            crate::services::agent_local::session_limits::CURRENT_SESSION_SCHEMA_VERSION,
        id: "abc-123".into(),
        name: "Test".into(),
        created_at: now,
        updated_at: None,
        archived_at: None,
        pinned_at: None,
        model: "llama3".into(),
        provider: "ollama".into(),
        thinking_enabled: false,
        fast_mode_enabled: false,
        reasoning_mode: None,
        preserve_reasoning: Default::default(),
        accumulated_tokens: 0,
        context_tokens: None,
        messages: Vec::<AgentMessage>::new(),
        todos: vec![],
        todo_neglect_count: 0,
        todo_runs: vec![],
        active_todo_run_id: None,
        stream_failures: vec![],
        diagnostic_runs: vec![],
        plan_mode_enabled: true,
        plan_runs: vec![AgentPlanRun {
            id: "plan-1".into(),
            title: "Plan".into(),
            status: AgentPlanStatus::AwaitingApproval,
            path: "plan.md".into(),
            created_at: now,
            updated_at: now,
        }],
        active_plan_id: Some("plan-1".into()),
        plan_workflow_status: AgentPlanWorkflowStatus::AwaitingApproval,
        is_heartbeat: false,
        is_gateway: false,
        gateway_channel_key: None,
        project_id: None,
        working_dir: String::new(),
        working_dir_managed: false,
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
