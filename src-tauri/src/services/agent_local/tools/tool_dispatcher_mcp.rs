use crate::services::agent_local::types_tools::ToolResult;
use serde_json::Value;

pub async fn dispatch_mcp(tool_name: &str, args: &Value) -> Option<ToolResult> {
    match tool_name {
        "search_mcp_tools" => Some(super::tool_mcp::execute(args).await),
        _ => None,
    }
}
