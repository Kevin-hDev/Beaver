use super::stream_events::AgentEventEmitter;
use super::subagent_tool_profile::SubagentToolProfile;
use super::tool_dispatch_trace::DispatchTrace;
use super::types_tools::ToolResult;
use serde_json::Value;
use std::path::Path;
use tokio_util::sync::CancellationToken;

#[derive(Clone)]
pub struct ToolDispatchAuthority {
    pub on_event: AgentEventEmitter,
    pub permission_mode: String,
    pub plan_active: bool,
}

#[allow(clippy::too_many_arguments)]
pub async fn dispatch(
    tool_name: &str,
    args: &Value,
    working_dir: &Path,
    trace: DispatchTrace<'_>,
    cancel: CancellationToken,
    profile: Option<SubagentToolProfile>,
    authority: &ToolDispatchAuthority,
) -> ToolResult {
    let Some(request_id) = trace.request_id else {
        return crate::services::extensions::unavailable_tool_result();
    };
    let purpose = crate::services::llm::request_purpose::RequestPurpose::for_request(
        trace.session_id,
        request_id,
    )
    .await;
    let scope = crate::services::extensions::core_scope::AgentCoreScope {
        session_id: trace.session_id.to_string(),
        request_id: request_id.to_string(),
        working_directory: working_dir.to_path_buf(),
        permission_mode: authority.permission_mode.clone(),
        profile,
        purpose,
        cancel,
        on_event: authority.on_event.clone(),
        plan_active: authority.plan_active,
    };
    crate::services::extensions::dispatch_tool(tool_name, args, scope)
        .await
        .unwrap_or_else(crate::services::extensions::unavailable_tool_result)
}
