use serde_json::Value;

/// Both Plan mode tool definitions (group `plan_mode`), enabled by default.
pub fn plan_tool_definitions() -> Vec<Value> {
    vec![planmode_definition(), exitplanmode_definition()]
}

pub fn planmode_definition() -> Value {
    super::tool_definitions::tool_def(
        "planmode",
        "Publish or update the implementation plan while Plan mode is active. \
         Plan mode lets you explore the codebase read-only and design an approach before any code is written. \
         Use this tool only after: \
         1. Read-only exploration is complete (read_file, grep, glob, list_dir). \
         2. Every important design question has been answered (use ask_user_choice first if needed). \
         This tool asks the user for final approval itself and returns their decision. Do not assume approval — wait for the return value. \
         On approval, immediately call exitplanmode to leave Plan mode and start implementation. \
         On rejection, revise the plan or ask clarifying questions, then re-publish.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "title": {"type": "string", "description": "Short plan title"},
                "content": {"type": "string", "description": "Markdown plan content"}
            },
            "required": ["title", "content"]
        }),
    )
}

pub fn exitplanmode_definition() -> Value {
    super::tool_definitions::tool_def(
        "exitplanmode",
        "Exit Plan mode after the user has approved the plan via planmode. \
         Call this only when planmode returned the approved decision. On success, immediately start implementing the plan. \
         If the plan was rejected, do not call this tool — revise and re-publish via planmode.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "status": {
                    "type": "string",
                    "enum": ["approved", "rejected"],
                    "description": "Final plan decision"
                }
            },
            "required": ["status"]
        }),
    )
}
