use super::*;

#[test]
fn tool_identifiers_are_bounded_and_reject_controls() {
    assert_eq!(
        bounded(Some("call_123"), MAX_TOOL_ID_BYTES).unwrap(),
        "call_123"
    );
    assert!(bounded(None, MAX_TOOL_ID_BYTES).is_err());
    assert!(bounded(Some(""), MAX_TOOL_ID_BYTES).is_err());
    assert!(bounded(Some("bad\nname"), MAX_TOOL_NAME_BYTES).is_err());
    assert!(bounded(Some(&"x".repeat(MAX_TOOL_ID_BYTES + 1)), MAX_TOOL_ID_BYTES).is_err());
}

#[test]
fn tool_calls_are_bounded() {
    let mut result = StreamResult::default();
    assert!(has_tool_capacity(&result));

    result.tool_calls = (0..MAX_TOOL_CALLS)
        .map(|index| (format!("tool_{index}"), serde_json::json!({})))
        .collect();

    assert!(!has_tool_capacity(&result));
}
