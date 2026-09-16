use serde_json::Value;
use std::path::Path;

use super::tool_dispatcher;
use super::stream_events::AgentEventEmitter;
use super::types_tools::ToolResult;

#[expect(
    clippy::too_many_arguments,
    reason = "parallel dispatch keeps permission and turn context explicit"
)]
pub async fn dispatch_read(
    on_event: &AgentEventEmitter,
    name: &str,
    args: &Value,
    working_dir: &Path,
    session_id: &str,
    request_id: &str,
    tool_call_id: Option<&str>,
    cancel: tokio_util::sync::CancellationToken,
    permission_mode: &str,
    plan_active: bool,
) -> ToolResult {
    super::tool_executor_diagnostics::started(session_id, name, args, working_dir).await;
    tool_dispatcher::dispatch_authorized_with_progress(
        name,
        args,
        working_dir,
        super::tool_dispatch_trace::DispatchTrace {
            session_id,
            request_id: Some(request_id),
            tool_call_id,
        },
        cancel,
        permission_mode == "chat",
        None,
        tool_dispatcher::ToolDispatchAuthority {
            on_event: on_event.clone(),
            permission_mode: permission_mode.to_string(),
            plan_active,
        },
    )
    .await
}
