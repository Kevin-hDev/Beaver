use serde_json::Value;

pub fn get_tool_definitions() -> Vec<Value> {
    let mut defs = Vec::new();
    defs.extend(super::tool_definitions_core::core_tool_definitions());
    defs.extend(super::tool_definitions_search::search_tool_definitions());
    defs.extend(super::tool_definitions_web::web_tool_definitions());
    defs.extend(super::tool_definitions_skills::skill_tool_definitions());
    defs.extend(super::tool_definitions_git::git_tool_definitions());
    defs.extend(super::tool_definitions_todo::todo_and_diagnostics_definitions());
    defs.push(super::tool_definitions_interactive::ask_user_choice_definition());
    defs.extend(super::tool_definitions_plan::plan_tool_definitions());
    defs.push(super::tool_definitions_subagent::delegate_task_definition());
    defs.extend(super::tool_definitions_subagent::subagent_control_definitions());
    defs.extend(super::tool_definitions_subagent::subagent_change_definitions());
    defs.extend(super::tool_definitions_forecast::forecast_tool_definitions());
    defs.extend(super::tool_definitions_office::office_tool_definitions());
    defs.extend(super::tool_definitions_mcp::mcp_tool_definitions());
    defs
}

/// Build a single OpenAI-style function tool definition.
pub(in crate::services::agent_local) fn tool_def(
    name: &str,
    description: &str,
    parameters: Value,
) -> Value {
    serde_json::json!({
        "type": "function",
        "function": { "name": name, "description": description, "parameters": parameters }
    })
}
