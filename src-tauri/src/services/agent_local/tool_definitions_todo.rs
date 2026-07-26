use serde_json::Value;

/// All todo + diagnostics tool definitions (groups `todo_list` and `agent_diagnostics`).
pub fn todo_and_diagnostics_definitions() -> Vec<Value> {
    vec![
        todo_write_definition(),
        todo_history_definition(),
        todo_pause_definition(),
        todo_resume_definition(),
        todo_delete_definition(),
        agent_diagnostics_definition(),
    ]
}

fn todo_write_definition() -> Value {
    super::tool_definitions::tool_def(
        "todo_write",
        "Create or update the current task checklist. Use this for multi-step coding tasks. \
         Send the full list each time, with at most one task marked in_progress.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "todos": {
                    "type": "array",
                    "maxItems": 50,
                    "items": {
                        "type": "object",
                        "properties": {
                            "content": {"type": "string", "description": "Short task name"},
                            "active_form": {"type": "string", "description": "Short present-tense label for an in-progress task"},
                            "status": {"type": "string", "enum": ["pending", "in_progress", "completed"]}
                        },
                        "required": ["content", "status"]
                    }
                }
            },
            "required": ["todos"]
        }),
    )
}

fn todo_history_definition() -> Value {
    super::tool_definitions::tool_def(
        "todo_history",
        "List saved todo checklists for this session. Hidden from the user UI.",
        serde_json::json!({
            "type": "object",
            "properties": {}
        }),
    )
}

fn todo_pause_definition() -> Value {
    super::tool_definitions::tool_def(
        "todo_pause",
        "Pause the active checklist before switching to another task or diagnostic.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "reason": {"type": "string", "description": "Short reason for pausing"}
            }
        }),
    )
}

fn todo_resume_definition() -> Value {
    super::tool_definitions::tool_def(
        "todo_resume",
        "Resume a saved checklist by id and make it visible as the active todo.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "id": {"type": "string", "description": "Todo checklist id from Todo memory or todo_history"}
            },
            "required": ["id"]
        }),
    )
}

fn todo_delete_definition() -> Value {
    super::tool_definitions::tool_def(
        "todo_delete",
        "Delete a checklist only when it should not be resumed later. Provide exactly one of id or active=true. \
         active=true deletes only the current active checklist. To delete paused checklists, use ids from Todo memory or todo_history. Hidden from the user UI.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "id": {"type": "string", "description": "Todo checklist id from Todo memory or todo_history. Required for paused checklists."},
                "active": {"type": "boolean", "description": "Set true only to delete the current active checklist, never paused checklists."}
            }
        }),
    )
}

fn agent_diagnostics_definition() -> Value {
    super::tool_definitions::tool_def(
        "agent_diagnostics",
        "Read recent safe stream diagnostics for this session. Hidden from the user UI.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "limit": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 20,
                    "description": "How many recent non-diagnostic tool calls to include. Defaults to 1."
                }
            }
        }),
    )
}
