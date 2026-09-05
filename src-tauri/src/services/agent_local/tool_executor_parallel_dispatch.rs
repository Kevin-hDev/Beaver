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
    cancel: tokio_util::sync::CancellationToken,
    chat_mode: bool,
) -> ToolResult {
    super::tool_executor_diagnostics::started(session_id, name, args, working_dir).await;
    tool_dispatcher::dispatch_for_mode(
        name,
        args,
        working_dir,
        session_id,
        Some(request_id),
        cancel,
        chat_mode,
    )
    .await
}
