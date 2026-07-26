use super::tool_todo_parse::parse_todos;
use super::tool_todo_state::apply_todos_to_session;
use super::types_session::AgentSession;
use super::types_todo::{AgentTodoItem, AgentTodoRun, AgentTodoRunStatus, AgentTodoStatus};
use chrono::Utc;
use serde_json::json;

#[test]
fn apply_todos_creates_new_run_when_match_is_too_weak() {
    let mut session = test_session();
    let first = parse_todos(&json!({
        "todos": [
            {"content": "A", "status": "pending"},
            {"content": "B", "status": "pending"},
            {"content": "Shared", "status": "pending"}
        ]
    }))
    .unwrap();
    apply_todos_to_session(&mut session, first);
    let second = parse_todos(&json!({
        "todos": [
            {"content": "X", "status": "pending"},
            {"content": "Y", "status": "pending"},
            {"content": "Shared", "status": "pending"}
        ]
    }))
    .unwrap();

    apply_todos_to_session(&mut session, second);

    assert_eq!(session.todo_runs.len(), 2);
    assert_eq!(session.todo_runs[0].status, AgentTodoRunStatus::Paused);
}

#[test]
fn reminder_includes_active_todo_items() {
    let mut session = test_session();
    apply_todos_to_session(
        &mut session,
        parse_todos(&json!({
            "todos": [
                {"content": "Lire", "status": "completed"},
                {"content": "Coder", "status": "in_progress"}
            ]
        }))
        .unwrap(),
    );
    let active_id = session.active_todo_run_id.clone().unwrap();

    let reminder = super::tool_todo_summary::reminder(&session).unwrap();

    assert!(reminder.contains("Active todo"));
    assert!(reminder.contains(&format!("id={active_id}")));
    assert!(reminder.contains("[completed] Lire"));
    assert!(reminder.contains("[in_progress] Coder"));
}

#[test]
fn reminder_lists_all_paused_todos() {
    let mut session = test_session();
    session.todo_runs.push(paused_run(
        "11111111-1111-4111-8111-111111111111",
        "Setup CI",
    ));
    session.todo_runs.push(paused_run(
        "22222222-2222-4222-8222-222222222222",
        "Doc API",
    ));

    let reminder = super::tool_todo_summary::reminder(&session).unwrap();

    assert!(reminder.contains("Paused todos (2)"));
    assert!(reminder.contains("Setup CI"));
    assert!(reminder.contains("Doc API"));
}

#[test]
fn reminder_escalates_stale_active_todo() {
    let mut session = test_session();
    apply_todos_to_session(
        &mut session,
        parse_todos(&json!({
            "todos": [{"content": "Coder", "status": "in_progress"}]
        }))
        .unwrap(),
    );
    session.todo_neglect_count = super::tool_todo_neglect::TODO_NEGLECT_ESCALATE_AFTER;

    let reminder = super::tool_todo_summary::reminder(&session).unwrap();

    assert!(reminder.contains("has not been updated"));
}

#[test]
fn user_turn_auto_pauses_stale_active_todo() {
    let mut session = test_session();
    apply_todos_to_session(
        &mut session,
        parse_todos(&json!({
            "todos": [{"content": "Coder", "status": "in_progress"}]
        }))
        .unwrap(),
    );
    session.todo_neglect_count = super::tool_todo_neglect::TODO_NEGLECT_AUTO_PAUSE_AFTER - 1;

    let paused = super::tool_todo_neglect::record_user_turn(&mut session);

    assert!(paused);
    assert!(session.todos.is_empty());
    assert!(session.active_todo_run_id.is_none());
    assert_eq!(session.todo_neglect_count, 0);
    assert_eq!(session.todo_runs[0].status, AgentTodoRunStatus::Paused);
}

#[test]
fn legacy_session_defaults_missing_neglect_count() {
    let json = json!({
        "id": "abc-123",
        "name": "Legacy",
        "created_at": Utc::now(),
        "model": "llama3",
        "provider": "ollama",
        "thinking_enabled": false,
        "accumulated_tokens": 0,
        "messages": [],
        "working_dir": ""
    });

    let session: AgentSession = serde_json::from_value(json).unwrap();

    assert_eq!(session.todo_neglect_count, 0);
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

fn paused_run(id: &str, title: &str) -> AgentTodoRun {
    AgentTodoRun {
        id: id.into(),
        title: title.into(),
        status: AgentTodoRunStatus::Paused,
        todos: vec![AgentTodoItem {
            content: title.into(),
            active_form: None,
            status: AgentTodoStatus::Pending,
        }],
        created_at: Utc::now(),
        updated_at: Utc::now(),
        paused_reason: Some("Attente".into()),
    }
}
