use super::*;
use crate::services::agent_local::types_session::AgentSession;
use crate::services::agent_local::types_todo::{AgentTodoRunStatus, AgentTodoStatus};
use chrono::Utc;
use serde_json::json;

#[test]
fn parse_accepts_valid_todos() {
    let parsed = parse_todos(&json!({
        "todos": [
            {"content": "Lire le code", "status": "completed"},
            {"content": "Implémenter", "activeForm": "Implémente", "status": "in_progress"},
            {"content": "Tester", "status": "pending"}
        ]
    }))
    .unwrap();

    assert_eq!(parsed.len(), 3);
    assert_eq!(parsed[1].active_form.as_deref(), Some("Implémente"));
    assert_eq!(parsed[1].status, AgentTodoStatus::InProgress);
}

#[test]
fn parse_rejects_invalid_status() {
    let err = parse_todos(&json!({
        "todos": [{"content": "Lire", "status": "started"}]
    }))
    .unwrap_err();

    assert!(err.contains("status"));
}

#[test]
fn parse_rejects_more_than_fifty_todos() {
    let todos: Vec<_> = (0..51)
        .map(|i| json!({"content": format!("Tâche {i}"), "status": "pending"}))
        .collect();
    let err = parse_todos(&json!({ "todos": todos })).unwrap_err();

    assert!(err.contains("maximum 50"));
}

#[test]
fn parse_rejects_multiple_in_progress() {
    let err = parse_todos(&json!({
        "todos": [
            {"content": "A", "status": "in_progress"},
            {"content": "B", "status": "in_progress"}
        ]
    }))
    .unwrap_err();

    assert!(err.contains("une seule"));
}

#[test]
fn apply_todos_updates_session() {
    let mut session = test_session();
    let todos = parse_todos(&json!({
        "todos": [{"content": "Valider", "status": "completed"}]
    }))
    .unwrap();

    apply_todos_to_session(&mut session, todos);

    assert_eq!(session.todos.len(), 1);
    assert_eq!(session.todos[0].content, "Valider");
    assert_eq!(session.todo_runs.len(), 1);
    assert!(session.active_todo_run_id.is_some());
    assert_eq!(session.todo_neglect_count, 0);
}

#[test]
fn apply_todos_updates_existing_run_for_same_tasks() {
    let mut session = test_session();
    let first = parse_todos(&json!({
        "todos": [{"content": "Lire", "status": "pending"}]
    }))
    .unwrap();
    apply_todos_to_session(&mut session, first);
    let run_id = session.active_todo_run_id.clone();
    let second = parse_todos(&json!({
        "todos": [{"content": "Lire", "status": "completed"}]
    }))
    .unwrap();

    apply_todos_to_session(&mut session, second);

    assert_eq!(session.todo_runs.len(), 1);
    assert_eq!(session.active_todo_run_id, run_id);
    assert_eq!(session.todo_runs[0].status, AgentTodoRunStatus::Completed);
}

#[test]
fn apply_todos_auto_pauses_different_unfinished_run() {
    let mut session = test_session();
    let first = parse_todos(&json!({
        "todos": [{"content": "Implémenter feature", "status": "in_progress"}]
    }))
    .unwrap();
    apply_todos_to_session(&mut session, first);
    let first_id = session.active_todo_run_id.clone().unwrap();
    let second = parse_todos(&json!({
        "todos": [{"content": "Diagnostiquer erreur", "status": "in_progress"}]
    }))
    .unwrap();

    apply_todos_to_session(&mut session, second);

    assert_eq!(session.todo_runs.len(), 2);
    assert_ne!(
        session.active_todo_run_id.as_deref(),
        Some(first_id.as_str())
    );
    assert_eq!(session.todo_runs[0].status, AgentTodoRunStatus::Paused);
}

#[test]
fn pause_and_resume_restore_active_todos() {
    let mut session = test_session();
    let todos = parse_todos(&json!({
        "todos": [{"content": "Reprendre", "status": "pending"}]
    }))
    .unwrap();
    apply_todos_to_session(&mut session, todos);
    let run_id = session.active_todo_run_id.clone().unwrap();

    super::super::tool_todo_state::pause_active(&mut session, Some("Attente".into()));
    assert!(session.todos.is_empty());
    assert_eq!(session.todo_runs[0].status, AgentTodoRunStatus::Paused);

    let active = super::super::tool_todo_state::resume_run(&mut session, &run_id).unwrap();
    assert_eq!(active[0].content, "Reprendre");
    assert_eq!(session.active_todo_run_id.as_deref(), Some(run_id.as_str()));
}

#[test]
fn delete_paused_run_removes_it_from_history() {
    let mut session = test_session();
    let first = parse_todos(&json!({
        "todos": [{"content": "Ancienne tâche", "status": "in_progress"}]
    }))
    .unwrap();
    apply_todos_to_session(&mut session, first);
    let paused_id = session.active_todo_run_id.clone().unwrap();
    let second = parse_todos(&json!({
        "todos": [{"content": "Nouvelle tâche", "status": "in_progress"}]
    }))
    .unwrap();
    apply_todos_to_session(&mut session, second);

    let active = super::super::tool_todo_state::delete_run(&mut session, &paused_id).unwrap();

    assert_eq!(session.todo_runs.len(), 1);
    assert_eq!(active, session.todos);
    assert_ne!(
        session.active_todo_run_id.as_deref(),
        Some(paused_id.as_str())
    );
}

#[test]
fn delete_active_run_clears_visible_todos() {
    let mut session = test_session();
    let todos = parse_todos(&json!({
        "todos": [{"content": "Active", "status": "in_progress"}]
    }))
    .unwrap();
    apply_todos_to_session(&mut session, todos);
    let run_id = session.active_todo_run_id.clone().unwrap();

    let active = super::super::tool_todo_state::delete_run(&mut session, &run_id).unwrap();

    assert!(active.is_empty());
    assert!(session.todos.is_empty());
    assert!(session.active_todo_run_id.is_none());
    assert!(session.todo_runs.is_empty());
}

#[test]
fn delete_active_arg_clears_visible_todos() {
    let mut session = test_session();
    let todos = parse_todos(&json!({
        "todos": [{"content": "Active", "status": "in_progress"}]
    }))
    .unwrap();
    apply_todos_to_session(&mut session, todos);
    let run_id = session.active_todo_run_id.clone().unwrap();

    let (active, deleted_id, status) =
        super::delete_run_for_args(&mut session, &json!({"active": true})).unwrap();

    assert!(active.is_empty());
    assert_eq!(deleted_id, run_id);
    assert_eq!(status, "active");
    assert!(session.todos.is_empty());
    assert!(session.active_todo_run_id.is_none());
    assert!(session.todo_runs.is_empty());
}

#[test]
fn delete_active_arg_guides_when_only_paused_runs_exist() {
    let mut session = test_session();
    let first = parse_todos(&json!({
        "todos": [{"content": "Ancienne tâche", "status": "in_progress"}]
    }))
    .unwrap();
    apply_todos_to_session(&mut session, first);
    let paused_id = session.active_todo_run_id.clone().unwrap();
    super::super::tool_todo_state::pause_active(&mut session, Some("Pause".into()));

    let err = super::delete_run_for_args(&mut session, &json!({"active": true})).unwrap_err();

    assert!(err.contains("aucune todo active"));
    assert!(err.contains("todo list(s) en pause"));
    assert!(err.contains(&paused_id));
    assert!(err.contains("todo_history"));
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
