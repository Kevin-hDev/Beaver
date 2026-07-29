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
        "Create or update the current task checklist. Use it for coding work that takes three or more distinct steps. \
         When to use: the user gives several tasks at once; a task needs planning before it can start; you are beginning a multi-step implementation. \
         When not to use: a single edit, a question, a one-step fix, or ordinary conversation. \
         Call it again in the same turn a task becomes completed or the active task changes. \
         A checklist that is not updated as you go is worse than no checklist. \
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
        "Resume a saved checklist by id and make it visible as the active todo. \
         Resume a paused checklist when its context becomes relevant again rather than starting a new one.",
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

#[cfg(test)]
mod tests {
    fn description(definition: &serde_json::Value) -> String {
        definition["function"]["description"]
            .as_str()
            .unwrap_or_default()
            .to_string()
    }

    /// Usage rules live in the tool definition, not in the system prompt: the definition is
    /// only sent when the tool is enabled, and it reaches the model at the point of decision.
    #[test]
    fn todo_write_definition_carries_cadence_and_negative_cases() {
        let text = description(&super::todo_write_definition());

        assert!(text.contains("in the same turn a task becomes completed"));
        assert!(text.contains("When not to use"));
        assert!(text.contains("at most one task marked in_progress"));
    }

    #[test]
    fn todo_resume_definition_says_when_to_resume() {
        let text = description(&super::todo_resume_definition());

        assert!(text.contains("context becomes relevant again"));
    }
}
