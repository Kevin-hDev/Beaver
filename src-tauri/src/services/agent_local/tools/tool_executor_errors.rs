use tokio_util::sync::CancellationToken;

use super::tool_result_contract::ToolErrorCategory;
use super::types_tools::ToolResult;

pub(super) fn permission(message: impl Into<String>, code: &'static str) -> ToolResult {
    ToolResult::error(message, code, ToolErrorCategory::Permission, false)
}

pub(super) fn denied_or_cancelled(cancel: &CancellationToken) -> ToolResult {
    if cancel.is_cancelled() {
        ToolResult::cancelled("Annulé.")
    } else {
        permission("L'utilisateur a refusé cette action.", "user_denied_tool")
    }
}
