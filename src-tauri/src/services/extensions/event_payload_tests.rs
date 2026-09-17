#[test]
fn event_payload_never_contains_scope_or_content() {
    let draft = super::event_payload::tool(
        "tool.execution.finished",
        "session",
        "request",
        super::event_payload::ToolEventPayload {
            name: "bash",
            tool_call_id: Some("call-1"),
            status: "error",
            error_code: Some("shell_exit_nonzero"),
            truncated: true,
            domain: Some("project"),
        },
    );
    let keys = draft
        .payload
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(
        keys,
        [
            "domain",
            "errorCode",
            "name",
            "status",
            "toolCallId",
            "truncated"
        ]
    );
    let serialized = serde_json::to_string(&draft.payload).unwrap();
    assert!(!serialized.contains("content"));
    assert!(!serialized.contains("scope"));
}

#[test]
fn unknown_events_are_refused() {
    let router = super::event_delivery::EventRouter::default();
    assert!(!router.publish(super::event_payload::EventDraft {
        event: "unknown.event",
        flow_id: "request".into(),
        session_id: "session".into(),
        request_id: "request".into(),
        payload: serde_json::json!({}),
        terminal: false,
        starts_flow: false,
    }));
}

#[test]
fn future_owned_events_keep_distinct_bounded_flows() {
    let automation = super::event_payload::automation_status(
        "automation.execution.started",
        "session",
        "request",
        "automation",
        "running",
        None,
        true,
        false,
    );
    let subagent = super::event_payload::subagent_status(
        "session", "request", "child", "running", true, false,
    );
    assert_eq!(automation.payload["automationId"], "automation");
    assert_eq!(subagent.payload["subagentId"], "child");
    assert_ne!(automation.flow_id, subagent.flow_id);
}
