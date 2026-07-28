use super::*;

#[test]
fn recent_context_is_bounded_and_prefers_recent_user_messages() {
    let messages = vec![
        ChatMessage {
            role: "user".to_string(),
            content: "older".repeat(1000),
            ..Default::default()
        },
        ChatMessage {
            role: "assistant".to_string(),
            content: "ignored".to_string(),
            ..Default::default()
        },
        ChatMessage {
            role: "user".to_string(),
            content: "PowerPoint".to_string(),
            ..Default::default()
        },
    ];

    let context = recent_user_context(&messages);

    assert!(context.starts_with("PowerPoint"));
    assert!(context.chars().count() <= MAX_CONTEXT_QUERY_CHARS);
    assert!(!context.contains("ignored"));
}
