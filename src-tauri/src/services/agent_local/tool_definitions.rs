use serde_json::Value;

pub fn get_tool_definitions() -> Vec<Value> {
    crate::services::extensions::merge_tool_definitions(native_tool_definitions())
}

pub(super) fn native_tool_definitions() -> Vec<Value> {
    let mut defs = Vec::new();
    defs.extend(super::tool_definitions_core::core_tool_definitions());
    defs.extend(super::tool_definitions_search::search_tool_definitions());
    defs.extend(super::tool_definitions_web::web_tool_definitions());
    defs.extend(super::tool_definitions_skills::skill_tool_definitions());
    defs.push(super::tool_definitions_automation::automation_definition());
    defs.extend(super::tool_definitions_git::git_tool_definitions());
    defs.extend(super::tool_definitions_todo::todo_definitions());
    defs.push(super::tool_definitions_interactive::ask_user_choice_definition());
    defs.extend(super::tool_definitions_plan::plan_tool_definitions());
    defs.push(super::tool_definitions_subagent::delegate_task_definition());
    defs.extend(super::tool_definitions_subagent::subagent_control_definitions());
    defs.extend(super::tool_definitions_subagent::subagent_change_definitions());
    defs.extend(super::tool_definitions_forecast::forecast_tool_definitions());
    defs.extend(super::tool_definitions_office::office_tool_definitions());
    defs.extend(super::tool_definitions_mcp::mcp_tool_definitions());
    defs.push(super::tool_definitions_extensions::extension_listing_definition());
    defs.push(super::tool_definitions_extensions::extension_inspection_definition());
    defs.push(super::tool_definitions_extensions::extension_resource_definition());
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

#[cfg(test)]
mod tests {
    #[test]
    fn main_agent_receives_both_extension_discovery_tools() {
        let definitions = super::get_tool_definitions();
        let names = definitions
            .iter()
            .filter_map(|tool| {
                tool.pointer("/function/name")
                    .and_then(serde_json::Value::as_str)
            })
            .collect::<Vec<_>>();
        assert!(names.contains(&"list_extensions"));
        assert!(names.contains(&"inspect_extensions"));
        assert!(names.contains(&super::super::tool_extension_resource::NAME));
    }
}
