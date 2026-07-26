use serde_json::Value;

use super::tool_todo_parse::{optional_text, MAX_TODO_REASON_CHARS};
use super::types_todo::{AgentTodoItem, AgentTodoRunStatus};
use crate::services::agent_local::types_ollama::StreamEvent;
use crate::services::agent_local::types_tools::ToolResult;

pub(crate) use super::tool_todo_parse::parse_todos;
pub(crate) use super::tool_todo_state::apply_todos_to_session;

pub async fn execute(args: &Value, session_id: &str) -> ToolResult {
    let todos = match parse_todos(args) {
        Ok(items) => items,
        Err(err) => return ToolResult::err(err),
    };

    match save_with(session_id, |session| {
        apply_todos_to_session(session, todos.clone());
        session.todos.clone()
    })
    .await
    {
        Ok(active) => {
            emit_update(session_id, active);
            ToolResult::ok("Todo list mise à jour.")
        }
        Err(_) => ToolResult::err("Mise à jour de la todo impossible."),
    }
}

pub async fn execute_history(_args: &Value, session_id: &str) -> ToolResult {
    match load_session(session_id).await {
        Ok(session) => ToolResult::ok(super::tool_todo_summary::history_summary(&session)),
        Err(_) => ToolResult::err("Historique todo indisponible."),
    }
}

pub async fn execute_pause(args: &Value, session_id: &str) -> ToolResult {
    let reason = match optional_text(args, "reason", MAX_TODO_REASON_CHARS) {
        Ok(value) => value,
        Err(err) => return ToolResult::err(err),
    };
    match save_with(session_id, |session| {
        super::tool_todo_state::pause_active(session, reason.clone());
        session.todos.clone()
    })
    .await
    {
        Ok(active) => {
            emit_update(session_id, active);
            ToolResult::ok("Todo list mise de côté.")
        }
        Err(_) => ToolResult::err("Mise en pause de la todo impossible."),
    }
}

pub async fn execute_resume(args: &Value, session_id: &str) -> ToolResult {
    let Some(run_id) = args.get("id").and_then(Value::as_str) else {
        return ToolResult::err("paramètre 'id' requis");
    };
    match save_with(session_id, |session| {
        super::tool_todo_state::resume_run(session, run_id)
    })
    .await
    {
        Ok(Ok(active)) => {
            emit_update(session_id, active);
            ToolResult::ok("Todo list reprise.")
        }
        Ok(Err(err)) => ToolResult::err(err),
        Err(_) => ToolResult::err("Reprise de la todo impossible."),
    }
}

pub async fn execute_delete(args: &Value, session_id: &str) -> ToolResult {
    match save_with(session_id, |session| delete_run_for_args(session, args)).await {
        Ok(Ok((active, run_id, status))) => {
            emit_update(session_id, active);
            ToolResult::ok(format!("Todo list supprimée: id={run_id} status={status}."))
        }
        Ok(Err(err)) => ToolResult::err(err),
        Err(_) => ToolResult::err("Suppression de la todo impossible."),
    }
}

fn delete_run_for_args(
    session: &mut super::types_session::AgentSession,
    args: &Value,
) -> Result<(Vec<AgentTodoItem>, String, String), String> {
    let explicit_id = args.get("id").and_then(Value::as_str).map(str::to_string);
    let delete_active = args.get("active").and_then(Value::as_bool).unwrap_or(false);
    if explicit_id.is_some() && delete_active {
        return Err("utiliser soit 'id', soit active=true".to_string());
    }
    if explicit_id.is_none() && !delete_active {
        return Err("paramètre 'id' ou active=true requis".to_string());
    }
    let run_id = if delete_active {
        session
            .active_todo_run_id
            .clone()
            .ok_or_else(|| delete_active_missing_message(session))?
    } else {
        explicit_id.ok_or_else(|| "paramètre 'id' ou active=true requis".to_string())?
    };
    let status = session
        .todo_runs
        .iter()
        .find(|run| run.id == run_id)
        .map(|run| todo_run_status_label(run.status).to_string())
        .ok_or_else(|| "todo introuvable".to_string())?;
    let active = super::tool_todo_state::delete_run(session, &run_id)?;
    Ok((active, run_id, status))
}

fn delete_active_missing_message(session: &super::types_session::AgentSession) -> String {
    let paused: Vec<_> = session
        .todo_runs
        .iter()
        .filter(|run| run.status == AgentTodoRunStatus::Paused)
        .collect();
    if paused.is_empty() {
        return "aucune todo active à supprimer".to_string();
    }

    let ids = paused
        .iter()
        .map(|run| format!("id={} title=\"{}\"", run.id, run.title))
        .collect::<Vec<_>>()
        .join("; ");
    format!(
        "aucune todo active à supprimer. {count} todo list(s) en pause existent: {ids}. \
         Utilise todo_delete avec l'id exact de chaque todo en pause, ou todo_history pour relire la liste.",
        count = paused.len()
    )
}

pub async fn append_session_reminder(
    messages: &mut [super::types_ollama::ChatMessage],
    session_id: &str,
) {
    let Ok(session) = load_session(session_id).await else {
        return;
    };
    let Some(reminder) = super::tool_todo_summary::reminder(&session) else {
        return;
    };
    if let Some(system) = messages
        .first_mut()
        .filter(|message| message.role == "system")
    {
        system.content.push_str(&reminder);
    }
}

pub(crate) fn emit_update(session_id: &str, todos: Vec<AgentTodoItem>) {
    let Some(app) = super::app_handle_global::get() else {
        return;
    };
    let emitter = super::stream_events::AgentEventEmitter::new(app.clone(), session_id.to_string());
    let _ = emitter.send(StreamEvent::TodoUpdated { todos });
}

fn todo_run_status_label(status: AgentTodoRunStatus) -> &'static str {
    match status {
        AgentTodoRunStatus::Active => "active",
        AgentTodoRunStatus::Paused => "paused",
        AgentTodoRunStatus::Completed => "completed",
    }
}

async fn save_with<T>(
    session_id: &str,
    edit: impl FnOnce(&mut super::types_session::AgentSession) -> T,
) -> Result<T, String> {
    super::session_store::validate_session_id(session_id)?;
    let lock = super::session_store::lock_session(session_id).await;
    let _guard = lock.lock().await;
    let mut session = super::session_store::get(session_id).await?;
    let result = edit(&mut session);
    super::session_store::save(&session).await?;
    Ok(result)
}

async fn load_session(session_id: &str) -> Result<super::types_session::AgentSession, String> {
    super::session_store::validate_session_id(session_id)?;
    super::session_store::get(session_id).await
}

#[cfg(test)]
#[path = "tool_todo_tests.rs"]
mod tests;
