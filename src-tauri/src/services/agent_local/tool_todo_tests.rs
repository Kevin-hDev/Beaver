use super::test_support::test_session;
use super::*;
use crate::services::agent_local::types_todo::AgentTodoStatus;
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
