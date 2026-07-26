use chrono::{DateTime, Utc};
use std::collections::HashSet;

use super::types_session::AgentSession;
use super::types_todo::{
    all_completed, AgentTodoItem, AgentTodoRun, AgentTodoRunStatus, AgentTodoStatus,
};

pub const MAX_TODO_RUNS: usize = 20;

pub fn apply_todos_to_session(session: &mut AgentSession, todos: Vec<AgentTodoItem>) {
    migrate_legacy_todos(session);
    session.todo_neglect_count = 0;
    if todos.is_empty() {
        session.todos.clear();
        session.active_todo_run_id = None;
        return;
    }

    let now = Utc::now();
    let active_index = active_run_index(session);
    if let Some(index) =
        active_index.filter(|i| same_todo_list(&session.todo_runs[*i].todos, &todos))
    {
        update_run(&mut session.todo_runs[index], todos.clone(), now);
        session.todos = todos;
        prune_runs(session);
        return;
    }

    if let Some(index) = active_index {
        pause_run(
            &mut session.todo_runs[index],
            Some("Nouvelle todo démarrée".to_string()),
            now,
        );
    }
    create_active_run(session, todos, now);
}

pub fn pause_active(session: &mut AgentSession, reason: Option<String>) -> bool {
    migrate_legacy_todos(session);
    session.todo_neglect_count = 0;
    let Some(index) = active_run_index(session) else {
        session.todos.clear();
        session.active_todo_run_id = None;
        return false;
    };
    pause_run(&mut session.todo_runs[index], reason, Utc::now());
    session.todos.clear();
    session.active_todo_run_id = None;
    true
}

pub fn resume_run(session: &mut AgentSession, run_id: &str) -> Result<Vec<AgentTodoItem>, String> {
    validate_run_id(run_id)?;
    migrate_legacy_todos(session);
    session.todo_neglect_count = 0;
    let now = Utc::now();
    if let Some(index) = active_run_index(session) {
        pause_run(
            &mut session.todo_runs[index],
            Some("Todo remplacée".to_string()),
            now,
        );
    }
    let run = session
        .todo_runs
        .iter_mut()
        .find(|run| run.id == run_id)
        .ok_or_else(|| "todo introuvable".to_string())?;
    run.status = status_for_todos(&run.todos);
    run.paused_reason = None;
    run.updated_at = now;
    session.active_todo_run_id = Some(run.id.clone());
    session.todos = run.todos.clone();
    Ok(session.todos.clone())
}

pub fn delete_run(session: &mut AgentSession, run_id: &str) -> Result<Vec<AgentTodoItem>, String> {
    validate_run_id(run_id)?;
    migrate_legacy_todos(session);
    session.todo_neglect_count = 0;
    let index = session
        .todo_runs
        .iter()
        .position(|run| run.id == run_id)
        .ok_or_else(|| "todo introuvable".to_string())?;
    let was_active = session.active_todo_run_id.as_deref() == Some(run_id);
    session.todo_runs.remove(index);
    if was_active {
        session.active_todo_run_id = None;
        session.todos.clear();
    }
    Ok(session.todos.clone())
}

pub fn completed_count(todos: &[AgentTodoItem]) -> usize {
    todos
        .iter()
        .filter(|todo| todo.status == AgentTodoStatus::Completed)
        .count()
}

fn create_active_run(session: &mut AgentSession, todos: Vec<AgentTodoItem>, now: DateTime<Utc>) {
    let run = AgentTodoRun {
        id: uuid::Uuid::new_v4().to_string(),
        title: infer_title(&todos),
        status: status_for_todos(&todos),
        todos: todos.clone(),
        created_at: now,
        updated_at: now,
        paused_reason: None,
    };
    session.active_todo_run_id = Some(run.id.clone());
    session.todo_runs.push(run);
    session.todos = todos;
    prune_runs(session);
}

pub(super) fn migrate_legacy_todos(session: &mut AgentSession) {
    if !session.todo_runs.is_empty() || session.todos.is_empty() {
        return;
    }
    create_active_run(session, session.todos.clone(), Utc::now());
}

pub(super) fn active_run_index(session: &AgentSession) -> Option<usize> {
    let active_id = session.active_todo_run_id.as_ref()?;
    session
        .todo_runs
        .iter()
        .position(|run| &run.id == active_id)
}

fn update_run(run: &mut AgentTodoRun, todos: Vec<AgentTodoItem>, now: DateTime<Utc>) {
    run.title = infer_title(&todos);
    run.status = status_for_todos(&todos);
    run.todos = todos;
    run.updated_at = now;
    run.paused_reason = None;
}

pub(super) fn pause_run(run: &mut AgentTodoRun, reason: Option<String>, now: DateTime<Utc>) {
    if run.status != AgentTodoRunStatus::Completed {
        run.status = AgentTodoRunStatus::Paused;
    }
    run.updated_at = now;
    run.paused_reason = reason;
}

fn status_for_todos(todos: &[AgentTodoItem]) -> AgentTodoRunStatus {
    if all_completed(todos) {
        AgentTodoRunStatus::Completed
    } else {
        AgentTodoRunStatus::Active
    }
}

fn same_todo_list(current: &[AgentTodoItem], next: &[AgentTodoItem]) -> bool {
    if current.is_empty() || next.is_empty() {
        return false;
    }
    if normalize(&current[0].content) == normalize(&next[0].content) {
        return true;
    }
    let current_keys: HashSet<String> = current
        .iter()
        .map(|todo| normalize(&todo.content))
        .collect();
    let matches = next
        .iter()
        .filter(|todo| current_keys.contains(&normalize(&todo.content)))
        .count();
    matches * 2 >= current.len().max(next.len())
}

fn infer_title(todos: &[AgentTodoItem]) -> String {
    todos
        .iter()
        .find(|todo| todo.status == AgentTodoStatus::InProgress)
        .or_else(|| todos.first())
        .map(|todo| todo.content.clone())
        .unwrap_or_else(|| "Todo list".to_string())
}

fn normalize(value: &str) -> String {
    value.trim().to_lowercase()
}

fn prune_runs(session: &mut AgentSession) {
    while session.todo_runs.len() > MAX_TODO_RUNS {
        let active = session.active_todo_run_id.as_deref();
        let index = session
            .todo_runs
            .iter()
            .position(|run| Some(run.id.as_str()) != active)
            .unwrap_or(0);
        session.todo_runs.remove(index);
    }
}

fn validate_run_id(run_id: &str) -> Result<(), String> {
    uuid::Uuid::parse_str(run_id)
        .map(|_| ())
        .map_err(|_| "identifiant de todo invalide".to_string())
}
