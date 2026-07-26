use chrono::Utc;

use super::diagnostic_redaction;
use super::types_diagnostics::{AgentDiagnosticEvent, AgentDiagnosticRun, AgentDiagnosticTodo};
use super::types_session::AgentSession;
use super::types_todo::AgentTodoStatus;

pub(crate) const MAX_STREAM_FAILURES: usize = 20;
pub(crate) const MAX_DIAGNOSTIC_RUNS: usize = 20;
pub(crate) const MAX_DIAGNOSTIC_EVENTS: usize = 12;
const MAX_TEXT: usize = 200;

pub(crate) fn active_todo(session: &AgentSession) -> Option<AgentDiagnosticTodo> {
    let run = session
        .active_todo_run_id
        .as_ref()
        .and_then(|id| session.todo_runs.iter().find(|run| &run.id == id))?;
    let completed = run
        .todos
        .iter()
        .filter(|todo| todo.status == AgentTodoStatus::Completed)
        .count();
    let active_task = run.todos.iter().find_map(|todo| {
        (todo.status == AgentTodoStatus::InProgress)
            .then(|| todo.active_form.as_ref().unwrap_or(&todo.content).clone())
    });
    Some(AgentDiagnosticTodo {
        id: run.id.clone(),
        title: clip(&run.title),
        active_task: active_task.map(|value| clip(&value)),
        completed,
        total: run.todos.len(),
        progress: format!("{completed}/{}", run.todos.len()),
    })
}

pub(crate) fn push_event(
    run: &mut AgentDiagnosticRun,
    phase: &str,
    message: &str,
    tool_name: Option<&str>,
    error_type: Option<&str>,
) {
    run.updated_at = Utc::now();
    run.events
        .push(event(phase, message, tool_name, error_type));
    trim(&mut run.events, MAX_DIAGNOSTIC_EVENTS);
}

pub(crate) fn event(
    phase: &str,
    message: &str,
    tool_name: Option<&str>,
    error_type: Option<&str>,
) -> AgentDiagnosticEvent {
    AgentDiagnosticEvent {
        at: Utc::now(),
        phase: phase.to_string(),
        message: clip(message),
        tool_name: tool_name.map(clip),
        error_type: error_type.map(str::to_string),
    }
}

pub(crate) async fn update_run<F>(
    session_id: &str,
    request_id: &str,
    mut f: F,
) -> Result<(), String>
where
    F: FnMut(&mut AgentSession, &mut AgentDiagnosticRun),
{
    update_session(session_id, |session| {
        if let Some(idx) = find_run(session, request_id) {
            let mut run = session.diagnostic_runs.remove(idx);
            f(session, &mut run);
            session.diagnostic_runs.insert(idx, run);
        }
    })
    .await
}

pub(crate) async fn update_session<F>(session_id: &str, mut f: F) -> Result<(), String>
where
    F: FnMut(&mut AgentSession),
{
    super::session_store::validate_session_id(session_id)?;
    let lock = super::session_store::lock_session(session_id).await;
    let _guard = lock.lock().await;
    let mut session = super::session_store::get(session_id).await?;
    f(&mut session);
    super::session_store::save(&session).await
}

pub(crate) fn find_run(session: &AgentSession, request_id: &str) -> Option<usize> {
    session
        .diagnostic_runs
        .iter()
        .position(|run| run.request_id == request_id)
}

pub(crate) fn trim<T>(items: &mut Vec<T>, max: usize) {
    while items.len() > max {
        items.remove(0);
    }
}

pub(crate) fn clip(value: &str) -> String {
    let clean = diagnostic_redaction::redact_text(value).replace(['\n', '\r', '\t'], " ");
    let mut end = clean.len();
    for (count, (idx, _)) in clean.char_indices().enumerate() {
        if count == MAX_TEXT {
            end = idx;
            break;
        }
    }
    let mut out = clean[..end].to_string();
    if clean[end..].chars().next().is_some() {
        out.push_str("...");
    }
    out
}
