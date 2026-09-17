use serde::Serialize;

#[derive(Clone)]
pub(super) struct EventDraft {
    pub(super) event: &'static str,
    pub(super) flow_id: String,
    pub(super) session_id: String,
    pub(super) request_id: String,
    pub(super) payload: serde_json::Value,
    pub(super) terminal: bool,
    pub(super) starts_flow: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct EventEnvelope {
    pub(super) id: uuid::Uuid,
    pub(super) sequence: u64,
    pub(super) occurred_at: String,
    #[serde(rename = "type")]
    pub(super) event: &'static str,
    pub(super) session_id: String,
    pub(super) request_id: String,
    pub(super) payload: serde_json::Value,
}

impl EventEnvelope {
    pub(super) fn build(draft: EventDraft, sequence: u64) -> Option<Self> {
        let envelope = Self {
            id: uuid::Uuid::new_v4(),
            sequence,
            occurred_at: super::diagnostic_time::now(),
            event: draft.event,
            session_id: draft.session_id,
            request_id: draft.request_id,
            payload: draft.payload,
        };
        let bytes = serde_json::to_vec(&envelope).ok()?;
        (bytes.len() <= super::types::MAX_EVENT_BYTES).then_some(envelope)
    }
}

pub(super) fn turn_started(session_id: &str, request_id: &str) -> EventDraft {
    turn(
        session_id,
        request_id,
        "session.turn.started",
        "started",
        false,
    )
}

pub(super) fn turn_terminal(
    session_id: &str,
    request_id: &str,
    event: &'static str,
    status: &'static str,
) -> EventDraft {
    turn(session_id, request_id, event, status, true)
}

fn turn(
    session_id: &str,
    request_id: &str,
    event: &'static str,
    status: &'static str,
    terminal: bool,
) -> EventDraft {
    EventDraft {
        event,
        flow_id: request_id.to_string(),
        session_id: session_id.to_string(),
        request_id: request_id.to_string(),
        payload: serde_json::json!({"status": status}),
        terminal,
        starts_flow: event == "session.turn.started",
    }
}

pub(super) fn tool(
    event: &'static str,
    session_id: &str,
    request_id: &str,
    payload: ToolEventPayload<'_>,
) -> EventDraft {
    EventDraft {
        event,
        flow_id: request_id.to_string(),
        session_id: session_id.to_string(),
        request_id: request_id.to_string(),
        payload: serde_json::json!({
            "name": payload.name,
            "toolCallId": payload.tool_call_id,
            "status": payload.status,
            "errorCode": payload.error_code,
            "truncated": payload.truncated,
            "domain": payload.domain,
        }),
        terminal: false,
        starts_flow: false,
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "future producer contract stays explicit"
)]
pub(super) fn automation_status(
    event: &'static str,
    session_id: &str,
    request_id: &str,
    automation_id: &str,
    status: &str,
    error_code: Option<&str>,
    starts_flow: bool,
    terminal: bool,
) -> EventDraft {
    owned_status(
        event,
        session_id,
        request_id,
        format!("automation:{automation_id}"),
        "automationId",
        automation_id,
        status,
        error_code,
        starts_flow,
        terminal,
    )
}

pub(super) fn subagent_status(
    session_id: &str,
    request_id: &str,
    subagent_id: &str,
    status: &str,
    starts_flow: bool,
    terminal: bool,
) -> EventDraft {
    owned_status(
        "subagent.status.changed",
        session_id,
        request_id,
        format!("subagent:{subagent_id}"),
        "subagentId",
        subagent_id,
        status,
        None,
        starts_flow,
        terminal,
    )
}

#[expect(
    clippy::too_many_arguments,
    reason = "closed event envelope stays explicit"
)]
fn owned_status(
    event: &'static str,
    session_id: &str,
    request_id: &str,
    flow_id: String,
    id_field: &str,
    id: &str,
    status: &str,
    error_code: Option<&str>,
    starts_flow: bool,
    terminal: bool,
) -> EventDraft {
    let mut payload = serde_json::Map::new();
    payload.insert(
        id_field.to_string(),
        serde_json::Value::String(id.to_string()),
    );
    payload.insert(
        "status".to_string(),
        serde_json::Value::String(status.to_string()),
    );
    payload.insert(
        "errorCode".to_string(),
        error_code.map_or(serde_json::Value::Null, |code| {
            serde_json::Value::String(code.to_string())
        }),
    );
    EventDraft {
        event,
        flow_id,
        session_id: session_id.to_string(),
        request_id: request_id.to_string(),
        payload: serde_json::Value::Object(payload),
        terminal,
        starts_flow,
    }
}

pub(super) struct ToolEventPayload<'a> {
    pub(super) name: &'a str,
    pub(super) tool_call_id: Option<&'a str>,
    pub(super) status: &'a str,
    pub(super) error_code: Option<&'a str>,
    pub(super) truncated: bool,
    pub(super) domain: Option<&'a str>,
}
