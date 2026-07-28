use serde_json::{json, Value};

pub fn extension_discovery_definition() -> Value {
    json!({
        "type": "function",
        "function": {
            "name": crate::services::extensions::SEARCH_TOOL_NAME,
            "description": concat!(
                "Find tools supplied by enabled Beaver extensions when the currently available ",
                "tools do not cover the task. Matching typed tools become available on the next ",
                "model turn of this same request. Search before installing dependencies or ",
                "recreating an existing capability with Bash."
            ),
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
