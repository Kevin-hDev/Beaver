use super::StreamEvent;

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
