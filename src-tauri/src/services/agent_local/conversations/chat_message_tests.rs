use super::types_ollama::{ChatMessage, ToolCallFunction, ToolCallOllama};

#[test]
fn system_and_user_messages_cannot_carry_reasoning_or_tools() {
    for message in [
        ChatMessage::system("rules".to_owned()),
        ChatMessage::user("question".to_owned()),
    ] {
        assert!(message.display_thinking.is_none());
        assert!(message.continuation.is_none());
        assert!(message.tool_loop_reasoning.is_none());
        assert!(message.tool_calls.is_none());
        assert!(message.tool_call_id.is_none());
        assert!(message.tool_name.is_none());
    }
}

#[test]
fn assistant_requires_reasoning_and_tool_calls_at_construction() {
    let tool_calls = vec![ToolCallOllama {
        id: Some("call-1".to_owned()),
        extra_content: None,
        function: ToolCallFunction {
            name: "lookup".to_owned(),
            arguments: serde_json::json!({"query": "small"}),
        },
    }];

    let message = ChatMessage::assistant(
        "answer".to_owned(),
        Some("reasoning".to_owned()),
        None,
        Some("reasoning".to_owned()),
        Some(tool_calls.clone()),
    );

    assert_eq!(message.role, "assistant");
    assert_eq!(message.display_thinking.as_deref(), Some("reasoning"));
    assert!(message.continuation.is_none());
    assert_eq!(message.tool_loop_reasoning.as_deref(), Some("reasoning"));
    assert_eq!(message.tool_calls.unwrap()[0].id.as_deref(), Some("call-1"));
}

#[test]
fn tool_requires_call_id_and_name_at_construction() {
    let message = ChatMessage::tool(
        "result".to_owned(),
        Some("call-1".to_owned()),
        Some("lookup".to_owned()),
    );

    assert_eq!(message.role, "tool");
    assert_eq!(message.tool_call_id.as_deref(), Some("call-1"));
    assert_eq!(message.tool_name.as_deref(), Some("lookup"));
    assert!(message.tool_loop_reasoning.is_none());
}

#[test]
fn tool_preserves_explicitly_missing_provider_metadata() {
    let message = ChatMessage::tool("result".to_owned(), None, None);

    assert!(message.tool_call_id.is_none());
    assert!(message.tool_name.is_none());
}

#[test]
fn images_are_added_without_changing_message_role() {
    let message = ChatMessage::user("describe".to_owned()).with_images(vec!["base64".to_owned()]);

    assert_eq!(message.role, "user");
    assert_eq!(message.images, Some(vec!["base64".to_owned()]));
    assert!(message.display_thinking.is_none());
    assert!(message.continuation.is_none());
    assert!(message.tool_loop_reasoning.is_none());
}

#[test]
fn generic_serde_never_exposes_reasoning_fields() {
    let message = ChatMessage::assistant(
        "visible".into(),
        Some("display-only".into()),
        None,
        Some("current-run-only".into()),
        None,
    );

    let serialized = serde_json::to_string(&message).unwrap();
    assert!(!serialized.contains("display-only"));
    assert!(!serialized.contains("current-run-only"));
    assert!(!serialized.contains("display_thinking"));
    assert!(!serialized.contains("continuation"));
    assert!(!serialized.contains("tool_loop_reasoning"));
}
