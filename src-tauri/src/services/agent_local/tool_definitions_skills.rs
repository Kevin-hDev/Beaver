use serde_json::Value;

/// `load_skill` — optional tool (group `skills`), enabled by default.
pub fn skill_tool_definitions() -> Vec<Value> {
    use super::tool_definitions::tool_def;
    vec![tool_def(
        "load_skill",
        "Load a skill by its exact source-qualified ID. Skills bundle specialized instructions and workflows (code review, security audits, fuzzing, frontend design, etc.). \
         Available skill IDs are listed in the system reminders of the conversation. Only invoke a skill from that list — never guess an ID. \
         If a skill matches the user's request, invoke it BEFORE producing any other response about the task. \
         Do not invoke a skill that is already loaded in the current turn.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "skill_id": {"type": "string", "description": "Exact source-qualified skill ID from the available skills list"}
            },
            "required": ["skill_id"]
        }),
    )]
}
