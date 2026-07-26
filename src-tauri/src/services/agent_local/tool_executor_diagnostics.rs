use serde_json::Value;
use std::path::Path;

pub async fn started(
    session_id: &str,
    request_id: &str,
    name: &str,
    args: &Value,
    working_dir: &Path,
) -> Option<Value> {
    let summary = super::diagnostic_args::summarize(name, args, working_dir);
    super::stream_diagnostics::record_tool(
        session_id,
        request_id,
        name,
        "started",
        summary.clone(),
        false,
    )
    .await;
    super::subagent_activity::record_tool_started(session_id, name, summary.as_ref()).await;
    summary
}

pub async fn detected(
    session_id: &str,
    request_id: &str,
    name: &str,
    args: &Value,
    working_dir: &Path,
) {
    let summary = super::diagnostic_args::summarize(name, args, working_dir);
    super::stream_diagnostics::record_tool(
        session_id, request_id, name, "detected", summary, false,
    )
    .await;
}

pub async fn completed(
    session_id: &str,
    request_id: &str,
    name: &str,
    summary: Option<Value>,
    is_error: bool,
) {
    super::stream_diagnostics::record_tool(
        session_id,
        request_id,
        name,
        "completed",
        summary.clone(),
        is_error,
    )
    .await;
    super::subagent_activity::record_tool_completed(session_id, name, summary.as_ref(), is_error)
        .await;
}
