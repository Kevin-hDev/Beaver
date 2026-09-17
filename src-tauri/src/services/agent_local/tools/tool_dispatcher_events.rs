use super::tool_dispatch_trace::DispatchTrace;
use super::types_tools::ToolResult;

pub(super) struct StartedEvent {
    resolved_path: Option<String>,
}

pub(super) fn start(
    authorized: bool,
    trace: DispatchTrace<'_>,
    tool_name: &str,
    args: &serde_json::Value,
    working_dir: &std::path::Path,
) -> Option<StartedEvent> {
    let request_id = authorized.then_some(trace.request_id).flatten()?;
    crate::services::extensions::tool_started(
        trace.session_id,
        request_id,
        tool_name,
        trace.tool_call_id,
    );
    Some(StartedEvent {
        resolved_path: super::tool_executor_helpers::resolve_tool_path(
            tool_name,
            args,
            working_dir,
        ),
    })
}

pub(super) fn finish(
    started: Option<StartedEvent>,
    trace: DispatchTrace<'_>,
    tool_name: &str,
    result: &ToolResult,
) {
    let Some(started) = started else {
        return;
    };
    let Some(request_id) = trace.request_id else {
        return;
    };
    let domain = super::memory_tool::resolved_path_domain(started.resolved_path.as_deref());
    crate::services::extensions::tool_finished(
        trace.session_id,
        request_id,
        tool_name,
        trace.tool_call_id,
        result,
        domain.as_deref(),
    );
}
