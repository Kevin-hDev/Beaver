use serde_json::{json, Value};

pub fn extension_discovery_definition() -> Value {
    let catalog = crate::services::extensions::catalog_snapshot();
    let catalog_section = if catalog.text.is_empty() {
        String::new()
    } else {
        format!("\n\nEnabled plugin catalog:\n{}", catalog.text)
    };
    let description = format!(
        "Find a capability among enabled Beaver plugins when its typed tools are not currently \
         available. A matching plugin is loaded as a complete unit for the next model turn and \
         remains loaded for this session. Search before installing dependencies or recreating an \
         existing capability with Bash.{catalog_section}"
    );
    json!({
        "type": "function",
        "function": {
            "name": crate::services::extensions::SEARCH_TOOL_NAME,
            "description": description,
            "parameters": {
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "A concise capability, product, file format, or action to find."
                    }
                },
                "required": ["query"],
                "additionalProperties": false
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absent_catalog_keeps_the_locked_search_contract_without_phantom_entries() {
        if crate::services::extensions::catalog_snapshot()
            .ordered_plugin_ids
            .is_empty()
        {
            let definition = extension_discovery_definition();
            assert!(!definition["function"]["description"]
                .as_str()
                .unwrap_or_default()
                .contains("\n- "));
        }
    }
}
