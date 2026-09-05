use super::stream_events::AgentEventEmitter;
use super::tool_executor_write::execute_write;
use super::types_tools::ToolResult;
use super::write_guard::WriteGuard;
use serde_json::Value;
use std::path::Path;
use tokio_util::sync::CancellationToken;

pub async fn execute_tracked_write(
    on_event: &AgentEventEmitter,
    name: &str,
    args: &Value,
    ctx: WriteExecContext<'_>,
) -> ToolResult {
    super::tool_executor_diagnostics::started(ctx.session_id, name, args, ctx.working_dir).await;
    if let Err(msg) = super::tool_plan_guard::ensure_allowed_for_session(
        name,
        args,
        ctx.session_id,
        ctx.plan_mode_active,
    )
    .await
    {
        return super::tool_executor_errors::permission(msg, "tool_not_allowed_in_plan");
    }
    let result = execute_write(
        on_event,
        name,
        args,
        ctx.working_dir,
        ctx.mode,
        ctx.write_guard,
        ctx.session_id,
        ctx.request_id,
        ctx.cancel,
        ctx.plan_mode_active,
        Some(ctx.tool_call_index),
    )
    .await;
    result
}

pub struct WriteExecContext<'a> {
    pub working_dir: &'a Path,
    pub mode: &'a str,
    pub write_guard: &'a mut WriteGuard,
    pub session_id: &'a str,
    pub request_id: &'a str,
    pub cancel: CancellationToken,
    pub plan_mode_active: bool,
    pub tool_call_index: usize,
}
