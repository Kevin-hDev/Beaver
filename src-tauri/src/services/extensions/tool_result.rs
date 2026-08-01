use crate::services::agent_local::types_tools::ToolResult;

pub(crate) fn unavailable() -> ToolResult {
    ToolResult::unavailable("extension_unavailable", "Extension indisponible.", true)
}
