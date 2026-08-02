use serde_json::Value;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use super::types_tools::ToolResult;

static METRICS_WARNING_EMITTED: AtomicBool = AtomicBool::new(false);

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
    match super::tool_metrics::record(name, result).await {
        Ok(()) => METRICS_WARNING_EMITTED.store(false, Ordering::Relaxed),
        Err(_) if !METRICS_WARNING_EMITTED.swap(true, Ordering::Relaxed) => {
            eprintln!("[tool-metrics] telemetry unavailable");
        }
        Err(_) => {}
    }
    super::subagent_activity::record_tool_completed(
        session_id,
        name,
        summary.as_ref(),
        result.is_error,
    )
        .await;
}
