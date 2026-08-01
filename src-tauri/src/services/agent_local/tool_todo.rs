use serde_json::Value;

use super::tool_todo_parse::{optional_text, MAX_TODO_REASON_CHARS};
use super::types_todo::AgentTodoItem;
use crate::services::agent_local::types_ollama::StreamEvent;
use crate::services::agent_local::types_tools::ToolResult;

pub(crate) use super::tool_todo_parse::parse_todos;
pub(crate) use super::tool_todo_state::apply_todos_to_session;
#[cfg(test)]
pub(super) use super::tool_todo_delete::delete_run_for_args;

pub async fn execute(args: &Value, session_id: &str) -> ToolResult {
    let todos = match parse_todos(args) {
        Ok(items) => items,
        Err(error) => return ToolResult::validation("todo_list_invalid", error),
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
        Err(_) => todo_save_failure(
            "todo_update_failed",
            "Mise à jour de la todo impossible.",
        ),
    }
}

pub async fn execute_history(_args: &Value, session_id: &str) -> ToolResult {
    match load_session(session_id).await {
        Ok(session) => ToolResult::ok(super::tool_todo_summary::history_summary(&session)),
        Err(_) => ToolResult::internal(
            "todo_history_unavailable",
            "Historique todo indisponible.",
            true,
        ),
    }
}

pub async fn execute_pause(args: &Value, session_id: &str) -> ToolResult {
    let reason = match optional_text(args, "reason", MAX_TODO_REASON_CHARS) {
        Ok(value) => value,
        Err(error) => return ToolResult::validation("todo_pause_reason_invalid", error),
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
        Err(_) => todo_save_failure(
            "todo_pause_failed",
            "Mise en pause de la todo impossible.",
        ),
    }
}

pub async fn execute_resume(args: &Value, session_id: &str) -> ToolResult {
    let Some(run_id) = args.get("id").and_then(Value::as_str) else {
        return ToolResult::validation("todo_id_required", "paramètre 'id' requis");
    };
    if uuid::Uuid::parse_str(run_id).is_err() {
        return ToolResult::validation("todo_id_invalid", "identifiant de todo invalide");
    }
    match save_with(session_id, |session| {
        super::tool_todo_state::resume_run(session, run_id)
    })
    .await
    {
        Ok(Ok(active)) => {
            emit_update(session_id, active);
            ToolResult::ok("Todo list reprise.")
        }
        Ok(Err(error)) => ToolResult::not_found("todo_not_found", error),
        Err(_) => todo_save_failure("todo_resume_failed", "Reprise de la todo impossible."),
    }
}

pub async fn execute_delete(args: &Value, session_id: &str) -> ToolResult {
    super::tool_todo_delete::execute(args, session_id).await
}

pub(super) fn todo_save_failure(code: &'static str, message: &'static str) -> ToolResult {
    ToolResult::internal(code, message, false)
        .with_error_hint("Relire todo_history avant de répéter cette modification.")
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

pub(super) async fn save_with<T>(
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
#[path = "tool_todo_history_tests.rs"]
mod history_tests;
#[cfg(test)]
#[path = "tool_todo_test_support.rs"]
mod test_support;
#[cfg(test)]
#[path = "tool_todo_tests.rs"]
mod tests;
