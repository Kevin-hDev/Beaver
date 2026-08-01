use serde_json::Value;

use super::types_todo::{AgentTodoItem, AgentTodoRunStatus};
use super::types_tools::ToolResult;

pub(super) async fn execute(args: &Value, session_id: &str) -> ToolResult {
    match super::tool_todo::save_with(session_id, |session| {
        delete_run_for_args(session, args)
    })
    .await
    {
        Ok(Ok((active, run_id, status))) => {
            super::tool_todo::emit_update(session_id, active);
            ToolResult::ok(format!("Todo list supprimée: id={run_id} status={status}."))
        }
        Ok(Err(error)) => error,
        Err(_) => super::tool_todo::todo_save_failure(
            "todo_delete_failed",
            "Suppression de la todo impossible.",
        ),
    }
}

pub(super) fn delete_run_for_args(
    session: &mut super::types_session::AgentSession,
    args: &Value,
) -> Result<(Vec<AgentTodoItem>, String, String), ToolResult> {
    let explicit_id = args.get("id").and_then(Value::as_str).map(str::to_string);
    let delete_active = args.get("active").and_then(Value::as_bool).unwrap_or(false);
    if explicit_id.is_some() && delete_active {
        return Err(ToolResult::validation(
            "todo_delete_selector_conflict",
            "utiliser soit 'id', soit active=true",
        ));
    }
    if explicit_id.is_none() && !delete_active {
        return Err(ToolResult::validation(
            "todo_delete_selector_required",
            "paramètre 'id' ou active=true requis",
        ));
    }
    let run_id = if delete_active {
        session.active_todo_run_id.clone().ok_or_else(|| {
            ToolResult::conflict(
                "todo_active_missing",
                delete_active_missing_message(session),
            )
        })?
    } else {
        let id = explicit_id.ok_or_else(|| {
            ToolResult::validation(
                "todo_delete_selector_required",
                "paramètre 'id' ou active=true requis",
            )
        })?;
        validated_id(id)?
    };
    let status = session
        .todo_runs
        .iter()
        .find(|run| run.id == run_id)
        .map(|run| status_label(run.status).to_string())
        .ok_or_else(|| ToolResult::not_found("todo_not_found", "todo introuvable"))?;
    let active = super::tool_todo_state::delete_run(session, &run_id)
        .map_err(|error| ToolResult::internal("todo_delete_state_failed", error, false))?;
    Ok((active, run_id, status))
}

fn validated_id(id: String) -> Result<String, ToolResult> {
    if uuid::Uuid::parse_str(&id).is_err() {
        return Err(ToolResult::validation(
            "todo_id_invalid",
            "identifiant de todo invalide",
        ));
    }
    Ok(id)
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

fn status_label(status: AgentTodoRunStatus) -> &'static str {
    match status {
        AgentTodoRunStatus::Active => "active",
        AgentTodoRunStatus::Paused => "paused",
        AgentTodoRunStatus::Completed => "completed",
    }
}
