use serde_json::Value;
use std::path::Path;

use super::tool_dispatcher;
use super::types_tools::ToolResult;

pub async fn dispatch_read(
    name: &str,
    args: &Value,
    working_dir: &Path,
    session_id: &str,
    request_id: &str,
) -> ToolResult {
    let summary =
        super::tool_executor_diagnostics::started(session_id, request_id, name, args, working_dir)
            .await;
    let result = tool_dispatcher::dispatch(
        name,
        args,
        working_dir,
        session_id,
        tokio_util::sync::CancellationToken::new(),
    )
    .await;
    super::tool_executor_diagnostics::completed(
        session_id,
        request_id,
        name,
        summary,
        result.is_error,
    )
    .await;
    result
}
