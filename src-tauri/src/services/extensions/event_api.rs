use super::event_payload::ToolEventPayload;

pub(crate) fn turn_started(session_id: &str, request_id: &str) -> bool {
    publish(super::event_payload::turn_started(session_id, request_id))
}

pub(crate) fn turn_terminal(
    session_id: &str,
    request_id: &str,
    event: &'static str,
    status: &'static str,
) -> bool {
    publish(super::event_payload::turn_terminal(
        session_id, request_id, event, status,
    ))
}

pub(crate) fn tool_started(
    session_id: &str,
    request_id: &str,
    name: &str,
    tool_call_id: Option<&str>,
) {
    let _ = publish(super::event_payload::tool(
        "tool.execution.started",
        session_id,
        request_id,
        ToolEventPayload {
            name,
            tool_call_id,
            status: "started",
            error_code: None,
            truncated: false,
            domain: None,
        },
    ));
}

pub(crate) fn tool_finished(
    session_id: &str,
    request_id: &str,
    name: &str,
    tool_call_id: Option<&str>,
    result: &crate::services::agent_local::types_tools::ToolResult,
    domain: Option<&str>,
) {
    let _ = publish(super::event_payload::tool(
        "tool.execution.finished",
        session_id,
        request_id,
        ToolEventPayload {
            name,
            tool_call_id,
            status: result.status.as_str(),
            error_code: result.error.as_ref().map(|error| error.code.as_ref()),
            truncated: result.truncated,
            domain,
        },
    ));
}

pub(crate) fn automation_event(
    session_id: &str,
    request_id: &str,
    automation_id: &str,
    started: bool,
    status: &str,
    error_code: Option<&str>,
) -> bool {
    publish(super::event_payload::automation_status(
        if started {
            "automation.execution.started"
        } else {
            "automation.execution.finished"
        },
        session_id,
        request_id,
        automation_id,
        status,
        error_code,
        started,
        !started,
    ))
}

#[allow(dead_code, reason = "producer is connected by the subagent lot B7")]
pub(crate) fn subagent_status_changed(
    session_id: &str,
    request_id: &str,
    subagent_id: &str,
    status: &str,
    starts_flow: bool,
    terminal: bool,
) -> bool {
    publish(super::event_payload::subagent_status(
        session_id,
        request_id,
        subagent_id,
        status,
        starts_flow,
        terminal,
    ))
}

fn publish(draft: super::event_payload::EventDraft) -> bool {
    super::runtime::global()
        .map(|runtime| runtime.work.event_router().publish(draft))
        .unwrap_or(false)
}
