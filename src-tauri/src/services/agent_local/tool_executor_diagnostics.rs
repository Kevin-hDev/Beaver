use serde_json::Value;
use std::path::Path;

use super::types_tools::ToolResult;

pub async fn started(
    session_id: &str,
    name: &str,
    args: &Value,
    working_dir: &Path,
) -> Option<Value> {
    let summary = super::diagnostic_args::summarize(name, args, working_dir);
    super::subagent_activity::record_tool_started(session_id, name, summary.as_ref()).await;
    summary
}

pub async fn completed(
    session_id: &str,
    request_id: &str,
    name: &str,
    summary: Option<Value>,
    result: &ToolResult,
) {
    super::stream_diagnostics::record_tool(
        session_id,
        request_id,
        name,
        "completed",
        summary.clone(),
        Some(result),
    )
    .await;
    super::subagent_activity::record_tool_completed(
        session_id,
        name,
        summary.as_ref(),
        result.is_error,
    )
        .await;
}
