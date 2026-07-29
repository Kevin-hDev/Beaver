use serde_json::Value;

/// Plan mode tool definition (group `plan_mode`), enabled by default.
pub fn plan_tool_definitions() -> Vec<Value> {
    vec![planmode_definition()]
}

pub fn planmode_definition() -> Value {
    super::tool_definitions::tool_def(
        "planmode",
        "Publish or update the implementation plan while Plan mode is active. \
         Plan mode lets you explore the codebase read-only and design an approach before any code is written. \
         Use this tool only after: \
         1. Read-only exploration is complete (read_file, grep, glob, list_dir). \
         2. Every important design question has been answered (ask the user first if needed). \
         This tool asks the user for final approval itself. Do not assume approval — wait for it to finish. \
         On approval, the backend closes Plan mode automatically and tells you to start implementation. \
         If the user requests adjustments, revise the plan and publish it again.",
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
