use serde_json::Value;

pub fn automation_definition() -> Value {
    super::tool_definitions::tool_def(
        "manage_automation",
        "List, create, update, or delete a scheduled agentic automation. An automation runs in the current project with the current model, only the exact selected tools, and up to eight exact skill IDs. Use create or update only after the user confirms the trigger, instruction, tools, skills, and active state. Delete requires confirm=true.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["list", "create", "update", "delete"],
                    "description": "Operation to perform."
                },
                "id": {"type": "string", "description": "Existing automation ID for update or delete."},
                "name": {"type": "string", "description": "Short automation name for create or update."},
                "description": {"type": "string", "description": "Optional concise purpose."},
                "prompt": {"type": "string", "description": "Bounded instruction executed at each trigger."},
                "schedule": {
                    "type": "object",
                    "description": "Trigger: {kind:'once',datetime:'YYYY-MM-DDTHH:MM'}, {kind:'daily',time:'HH:MM'}, or {kind:'weekly',weekday:0..6,time:'HH:MM'}."
                },
                "skill_ids": {
                    "type": "array",
                    "items": {"type": "string"},
                    "maxItems": 8,
                    "description": "Exact source-qualified skill IDs from the available skills list."
                },
                "tool_names": {
                    "type": "array",
                    "items": {"type": "string"},
                    "maxItems": 12,
                    "description": "Exact tool names needed by the automation. Interactive, planning, nested subagent, and automation-management tools are refused."
                },
                "active": {"type": "boolean", "description": "Whether the automation should run on schedule."},
                "confirm": {"type": "boolean", "description": "Set true only when the user explicitly confirms deletion."}
            },
            "required": ["action"]
        }),
    )
}
