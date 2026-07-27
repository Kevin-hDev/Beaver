use serde_json::Value;

pub fn filter_tool_definitions(
    definitions: Vec<Value>,
    enabled_optional_tools: &[String],
) -> Vec<Value> {
    definitions
        .into_iter()
        .filter(|definition| {
            super::tool_catalog::tool_name(definition)
                .as_deref()
                .is_some_and(|name| {
                    super::tool_availability::available(
                        super::tool_catalog::is_enabled(
                            name,
                            enabled_optional_tools,
                        ),
                        crate::services::extensions::is_dynamic_tool(name),
                        crate::services::extensions::is_replacement(name),
                    )
                })
        })
        .collect()
}
