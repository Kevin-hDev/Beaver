use super::StreamEvent;
use crate::services::agent_local::types_interactive::AgentInteractiveChoiceKind;

#[test]
fn tool_result_serializes_readable_display_summary() {
    let event = StreamEvent::ToolResult {
        name: "load_skill".to_string(),
        content: "Skill loaded".to_string(),
        is_error: false,
        truncated: false,
        display_summary: Some("context7-docs".to_string()),
        tool_call_index: 0,
        resolved_path: None,
        domain: None,
        affected_paths: Vec::new(),
        file_changes: Vec::new(),
        start_line: None,
    };

    let serialized = serde_json::to_value(event).expect("stream event should serialize");

    assert_eq!(
        serialized["data"]["displaySummary"],
        serde_json::json!("context7-docs")
    );
    assert!(serialized.to_string().contains("context7-docs"));
}

#[test]
fn plan_approval_kind_reaches_the_frontend() {
    let event = StreamEvent::InteractiveChoiceRequest {
        session_id: "session-1".into(),
        id: "choice-1".into(),
        kind: AgentInteractiveChoiceKind::PlanApproval,
        questions: vec![],
        current_index: 0,
        total: 1,
    };

    let serialized = serde_json::to_value(event).expect("stream event should serialize");

    assert_eq!(
        serialized["data"]["kind"],
        serde_json::json!("plan_approval")
    );
}

#[test]
fn context_usage_serializes_for_the_live_ring() {
    let event = StreamEvent::ContextUsage {
        input_tokens: 120,
        output_tokens: 8,
        context_tokens: 128,
        estimated: true,
    };

    let serialized = serde_json::to_value(event).expect("serialize context usage");

    assert_eq!(serialized["event"], "contextUsage");
    assert_eq!(serialized["data"]["contextTokens"], 128);
    assert_eq!(serialized["data"]["estimated"], true);
}
