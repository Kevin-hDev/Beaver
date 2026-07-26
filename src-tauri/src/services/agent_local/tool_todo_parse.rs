use serde_json::Value;

use super::types_todo::{AgentTodoItem, AgentTodoStatus};

pub const MAX_TODOS: usize = 50;
pub const MAX_TODO_TEXT_CHARS: usize = 180;
pub const MAX_TODO_REASON_CHARS: usize = 240;

pub fn parse_todos(args: &Value) -> Result<Vec<AgentTodoItem>, String> {
    let raw = args
        .get("todos")
        .and_then(Value::as_array)
        .ok_or_else(|| "paramètre 'todos' requis".to_string())?;
    if raw.len() > MAX_TODOS {
        return Err(format!("maximum {MAX_TODOS} tâches"));
    }

    let mut in_progress = 0usize;
    let mut todos = Vec::with_capacity(raw.len());
    for item in raw {
        let obj = item
            .as_object()
            .ok_or_else(|| "chaque tâche doit être un objet JSON".to_string())?;
        let content = clean_required_text(obj.get("content"), "content", MAX_TODO_TEXT_CHARS)?;
        let active_form = match obj.get("active_form").or_else(|| obj.get("activeForm")) {
            Some(Value::Null) | None => None,
            value => Some(clean_required_text(
                value,
                "active_form",
                MAX_TODO_TEXT_CHARS,
            )?),
        };
        let status = parse_status(obj.get("status"))?;
        if status == AgentTodoStatus::InProgress {
            in_progress += 1;
        }
        todos.push(AgentTodoItem {
            content,
            active_form,
            status,
        });
    }

    if in_progress > 1 {
        return Err("une seule tâche peut être en cours".to_string());
    }
    Ok(todos)
}

pub fn optional_text(args: &Value, key: &str, max_chars: usize) -> Result<Option<String>, String> {
    match args.get(key) {
        Some(Value::Null) | None => Ok(None),
        value => clean_required_text(value, key, max_chars).map(Some),
    }
}

fn clean_required_text(
    value: Option<&Value>,
    name: &str,
    max_chars: usize,
) -> Result<String, String> {
    let text = value
        .and_then(Value::as_str)
        .ok_or_else(|| format!("'{name}' doit être une chaîne"))?
        .trim();
    if text.is_empty() {
        return Err(format!("'{name}' ne peut pas être vide"));
    }
    if text.chars().count() > max_chars {
        return Err(format!("'{name}' est trop long"));
    }
    if text.chars().any(|ch| ch.is_control()) {
        return Err(format!("'{name}' contient des caractères invalides"));
    }
    Ok(text.to_string())
}

fn parse_status(value: Option<&Value>) -> Result<AgentTodoStatus, String> {
    match value.and_then(Value::as_str) {
        Some("pending") => Ok(AgentTodoStatus::Pending),
        Some("in_progress") => Ok(AgentTodoStatus::InProgress),
        Some("completed") => Ok(AgentTodoStatus::Completed),
        _ => Err("'status' doit valoir pending, in_progress ou completed".to_string()),
    }
}
