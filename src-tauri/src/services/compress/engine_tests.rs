use super::*;
use crate::services::agent_local::types_ollama::ChatMessage;

fn msg(role: &str, content: &str) -> ChatMessage {
    ChatMessage {
        role: role.to_string(),
        content: content.to_string(),
        images: None,
        tool_calls: None,
        tool_name: None,
        tool_call_id: None,
        reasoning_content: None,
    }
}

#[test]
fn should_auto_compress_disabled() {
    assert!(!should_auto_compress(false, 200_000, 131_072, 170_000, 85));
}

#[test]
fn should_auto_compress_small_context_when_threshold_reached() {
    assert!(should_auto_compress(true, 32_768, 32_768, 30_000, 85));
}

#[test]
fn should_auto_compress_unknown_context_disabled() {
    assert!(!should_auto_compress(true, 0, 0, 30_000, 85));
}

#[test]
fn should_auto_compress_under_threshold() {
    assert!(!should_auto_compress(true, 131_072, 32_768, 20_000, 85));
}

#[test]
fn should_auto_compress_over_threshold() {
    assert!(should_auto_compress(true, 131_072, 32_768, 28_000, 85));
}

#[test]
fn build_post_compression_messages_structure() {
    let result = build_post_compression_messages("Test summary", true);
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].role, "system");
    assert!(result[0].content.contains("Compression boundary"));
    assert_eq!(result[1].role, "user");
    assert!(result[1].content.contains("Test summary"));
    assert!(result[1].content.contains("without asking"));
}

#[test]
fn build_post_compression_messages_manual() {
    let result = build_post_compression_messages("Test", false);
    assert!(!result[1].content.contains("without asking"));
}

#[test]
fn prepare_messages_strips_system() {
    let msgs = vec![
        msg("system", "System prompt"),
        msg("user", "Hello"),
        msg("assistant", "Hi"),
    ];
    let prepared = prepare_messages_for_compression(&msgs);
    assert!(prepared.iter().all(|m| m.role != "system"));
    assert_eq!(prepared.len(), 2);
}

#[test]
fn prepare_messages_preserves_order() {
    let msgs = vec![
        msg("user", "First"),
        msg("assistant", "Second"),
        msg("user", "Third"),
    ];
    let prepared = prepare_messages_for_compression(&msgs);
    assert_eq!(prepared[0].content, "First");
    assert_eq!(prepared[2].content, "Third");
}

#[test]
fn apply_compression_preserves_system() {
    let mut messages = vec![
        msg("system", "You are Claude"),
        msg("user", "Hello"),
        msg("assistant", "Hi"),
        msg("user", "More"),
        msg("assistant", "Response"),
    ];
    let pre = apply_compression(&mut messages, "Summary text", false);
    assert_eq!(pre, 5);
    assert_eq!(messages[0].role, "system");
    assert_eq!(messages[0].content, "You are Claude");
    assert_eq!(messages.len(), 3); // system + boundary + summary
}

#[test]
fn apply_compression_without_system() {
    let mut messages = vec![msg("user", "Hello"), msg("assistant", "Hi")];
    let pre = apply_compression(&mut messages, "Summary", true);
    assert_eq!(pre, 2);
    assert_eq!(messages.len(), 2); // boundary + summary (no system)
    assert_eq!(messages[0].role, "system"); // boundary is system role
}

#[test]
fn build_request_excludes_system() {
    let messages = vec![msg("system", "Secret system prompt"), msg("user", "Hello")];
    let request = build_compression_request_content(&messages, None);
    assert!(request.iter().all(|m| m.content != "Secret system prompt"));
}

#[test]
fn build_request_adds_compression_prompt() {
    let messages = vec![msg("user", "Hello")];
    let request = build_compression_request_content(&messages, None);
    let last = request.last().unwrap();
    assert_eq!(last.role, "user");
    assert!(last.content.contains("CRITICAL"));
    assert!(last.content.contains("Primary Request"));
}

#[test]
fn build_request_with_custom_instructions() {
    let messages = vec![msg("user", "Hello")];
    let request = build_compression_request_content(&messages, Some("Focus on Rust"));
    let last = request.last().unwrap();
    assert!(last.content.contains("Focus on Rust"));
}
