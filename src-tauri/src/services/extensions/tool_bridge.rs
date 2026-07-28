use serde_json::{json, Value};

pub fn definitions() -> Vec<Value> {
    super::registry_index::dynamic_tools()
        .into_iter()
        .map(|tool| {
            json!({
                "type": "function",
                "function": {
                    "name": tool.name,
                    "description": tool.description,
                    "parameters": tool.parameters,
                }
            })
        })
        .collect()
}

pub fn merge_definitions(core: Vec<Value>) -> Vec<Value> {
    merge(core, definitions())
}

fn merge(mut core: Vec<Value>, extensions: Vec<Value>) -> Vec<Value> {
    for extension in extensions {
        let Some(name) = definition_name(&extension) else {
            continue;
        };
        if let Some(index) = core
            .iter()
            .position(|definition| definition_name(definition) == Some(name))
        {
            core[index] = extension;
        } else {
            core.push(extension);
        }
    }
    core
}

pub fn validate_arguments(tool_name: &str, arguments: &Value) -> Result<Value, String> {
    super::validation::request_payload(arguments)?;
    let tool = super::registry_index::dynamic_tool(tool_name)
        .ok_or_else(|| super::error_codes::TOOL_UNAVAILABLE.to_string())?;
    crate::services::mcp_bridge::arguments::validate(arguments, Some(&tool.parameters))
        .map_err(|_| super::error_codes::TOOL_ARGUMENTS_INVALID.to_string())?;
    Ok(arguments.clone())
}

fn definition_name(definition: &Value) -> Option<&str> {
    definition.get("function")?.get("name")?.as_str()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_explicit_replacement_wins_without_duplicate_names() {
        let core = vec![json!({
            "type": "function",
            "function": {
                "name": "web_search",
                "description": "core",
                "parameters": {"type": "object"}
            }
        })];
        let extension = vec![json!({
            "type": "function",
            "function": {
                "name": "web_search",
                "description": "extension",
                "parameters": {"type": "object"}
            }
        })];

        let merged = merge(core, extension);

        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0]["function"]["description"], "extension");
    }
}
