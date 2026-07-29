use super::test_support::test_session;
use super::*;
use crate::services::agent_local::types_todo::AgentTodoRunStatus;
use serde_json::json;

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
