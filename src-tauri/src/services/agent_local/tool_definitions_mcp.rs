use serde_json::Value;

pub fn mcp_tool_definitions() -> Vec<Value> {
    vec![serde_json::json!({
        "type": "function",
        "function": {
            "name": "search_mcp_tools",
            "description": concat!(
                "Access external services (Canva, Linear, Slack, Vercel, Sentry, Apify, Notion, etc.) ",
                "via MCP (Model Context Protocol). Two-step workflow:\n",
                "1. SEARCH: Find available tools with {\"mode\":\"search\",\"query\":\"keyword\"}\n",
                "   Example: {\"mode\":\"search\",\"query\":\"issues\"} → returns tool IDs like linear.list_issues\n",
                "   Use empty query \"\" to list ALL available tools.\n",
                "2. CALL: Execute a found tool with {\"mode\":\"call\",\"tool_id\":\"service.tool_name\",\"arguments\":{...}}\n",
                "   Example: {\"mode\":\"call\",\"tool_id\":\"linear.list_issues\",\"arguments\":{\"limit\":5}}\n",
                "IMPORTANT: Pass mode, query, tool_id, arguments as direct top-level properties. ",
                "Do NOT wrap them in an 'input' field or stringify them."
            ),
            "parameters": {
                "type": "object",
                "properties": {
                    "mode": {
                        "type": "string",
                        "enum": ["search", "call"],
                        "description": "Required. 'search' to find tools, 'call' to execute a tool."
                    },
                    "query": {
                        "type": "string",
                        "description": "Mode search only. Keyword to filter tools by name/description. Use empty string for all tools."
                    },
                    "tool_id": {
                        "type": "string",
                        "description": "Mode call only. Format: 'service.tool_name' (e.g. 'linear.list_issues', 'canva.search-designs')."
                    },
                    "arguments": {
                        "type": "object",
                        "description": "Mode call only. Arguments to pass to the MCP tool as a JSON object."
                    }
                },
                "required": ["mode"]
            }
        }
    })]
}
