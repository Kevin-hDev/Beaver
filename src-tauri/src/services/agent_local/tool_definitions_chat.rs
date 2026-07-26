use serde_json::Value;

pub fn get_chat_tool_definitions() -> Vec<Value> {
    let mut defs = vec![
        super::tool_definitions_interactive::ask_user_choice_definition(),
        super::tool_definitions_plan::planmode_definition(),
        super::tool_definitions_plan::exitplanmode_definition(),
        super::tool_definitions::tool_def(
            "web_search",
            "Search the web for current information, documentation, or solutions.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string", "description": "Search query"}
                },
                "required": ["query"]
            }),
        ),
        super::tool_definitions::tool_def(
            "web_fetch",
            "Fetch and extract content from a URL.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "url": {"type": "string", "description": "URL to fetch"}
                },
                "required": ["url"]
            }),
        ),
    ];
    defs.extend(super::tool_definitions_mcp::mcp_tool_definitions());
    defs
}
