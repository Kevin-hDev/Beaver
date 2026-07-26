use super::types_tools::ToolResult;
use serde_json::Value;
use std::path::Path;
use tokio_util::sync::CancellationToken;

pub async fn dispatch(
    tool_name: &str,
    args: &Value,
    working_dir: &Path,
    session_id: &str,
    cancel: CancellationToken,
) -> ToolResult {
    if let Some(result) =
        super::tool_subagent_changes::dispatch(tool_name, args, working_dir, session_id).await
    {
        return result;
    }
    if let Some(result) =
        super::tool_subagent_control::dispatch(tool_name, args, session_id, cancel.clone()).await
    {
        return result;
    }
    if let Some(result) = super::tool_dispatcher_forecast::dispatch_forecast(
        tool_name,
        args,
        working_dir,
        session_id,
        cancel,
    )
    .await
    {
        return result;
    }
    if let Some(result) = super::tool_dispatcher_mcp::dispatch_mcp(tool_name, args).await {
        return result;
    }
    match super::tool_dispatcher_office::dispatch_office(
        tool_name,
        args,
        working_dir,
        session_id,
    )
    .await
    {
        Some(result) => result,
        None => ToolResult::err(format!("Outil inconnu: {tool_name}")),
    }
}
