use super::*;
use crate::services::agent_local::types_session::AgentSession;
use chrono::Utc;

#[tokio::test]
async fn running_child_has_pending_work() {
    let mut child = child("running");

    assert!(child_has_pending_work(&child).await);

    child.subagent_status = Some("completed".into());
    assert!(!child_has_pending_work(&child).await);
}

#[tokio::test]
async fn queued_prompt_keeps_completed_child_pending() {
    let mut child = child("completed");
    child.subagent_queued_prompts.push("suite".into());

    assert!(child_has_pending_work(&child).await);
}

#[test]
fn enqueue_prompt_marks_child_running() {
    let mut child = child("completed");

    let result = enqueue_prompt(&mut child, "suite");

    assert!(result.is_ok());
    assert_eq!(child.subagent_queued_prompts, vec!["suite"]);
    assert_eq!(child.subagent_status.as_deref(), Some("running"));
    assert!(child.updated_at.is_some());
}

#[test]
fn enqueue_prompt_deduplicates_spacing_without_changing_original() {
    let mut child = child("running");
    let original = "  corrige ceci\n\n```rust\n  let x = 1;\n```  ";

    assert!(enqueue_prompt(&mut child, original).is_ok());
    assert!(enqueue_prompt(
        &mut child,
        "corrige   ceci ```rust let x = 1; ```",
    )
    .is_ok());

    assert_eq!(child.subagent_queued_prompts, vec![original]);
}

#[test]
fn enqueue_prompt_rejects_full_queue() {
    let mut child = child("running");
    child
        .subagent_queued_prompts
        .extend((0..MAX_QUEUED_PROMPTS).map(|idx| format!("suite {idx}")));

    let result = enqueue_prompt(&mut child, "extra");

    assert!(result.is_err());
    assert_eq!(child.subagent_queued_prompts.len(), MAX_QUEUED_PROMPTS);
}

#[test]
fn enqueue_prompt_rejects_a_corrupt_duplicate_queue() {
    let mut child = child("running");
    child.subagent_queued_prompts = vec!["corrige".into(), "  corrige  ".into()];

    let result = enqueue_prompt(&mut child, "extra");

    assert!(result.is_err());
    assert_eq!(child.subagent_queued_prompts.len(), 2);
}

#[test]
fn archive_subagent_refuses_pending_child() {
    let result = can_archive_child(true);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().content, "Sous-agent encore actif.");
}

#[test]
fn archive_subagent_accepts_completed_child() {
    assert!(can_archive_child(false).is_ok());
}

#[test]
fn archive_subagent_requires_parent_ownership() {
    let mut agent = child("completed");

    assert!(is_owned_by_parent(&agent, "parent"));

    agent.parent_session_id = Some("other-parent".into());
    assert!(!is_owned_by_parent(&agent, "parent"));
}

fn child(status: &str) -> AgentSession {
    AgentSession {
        id: uuid::Uuid::new_v4().to_string(),
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
