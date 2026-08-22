use serde_json::Value;

pub fn automation_definition() -> Value {
    super::tool_definitions::tool_def(
        "manage_automation",
        "List, create, update, or delete a scheduled automation. Every automation runs through the complete Agent Local engine in full-access mode, with all currently enabled tools and skills. It uses the current model and optional current project. Use create or update only after the user confirms the trigger, instruction, and active state. Delete requires confirm=true.",
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
                "active": {"type": "boolean", "description": "Whether the automation should run on schedule."},
                "confirm": {"type": "boolean", "description": "Set true only when the user explicitly confirms deletion."}
            },
            "required": ["action"]
        }),
    )
}
