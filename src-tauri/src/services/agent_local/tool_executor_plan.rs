use crate::services::agent_local::types_tools::ToolResult;
use serde_json::Value;

pub async fn denied_with_summary(
    session_id: &str,
    request_id: &str,
    name: &str,
    msg: String,
    arg_summary: Option<Value>,
) -> ToolResult {
    let tr = super::tool_dispatcher::enrich_error(ToolResult::err(msg), name);
    super::tool_executor_diagnostics::completed(session_id, request_id, name, arg_summary, true)
        .await;
    tr
}

pub async fn denied_from_args(
    session_id: &str,
    request_id: &str,
    name: &str,
    msg: String,
    args: &Value,
    working_dir: &std::path::Path,
) -> ToolResult {
    let summary = super::diagnostic_args::summarize(name, args, working_dir);
    denied_with_summary(session_id, request_id, name, msg, summary).await
}
