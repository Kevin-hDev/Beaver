use super::context_capsules::recent_file_context_message;
use crate::services::agent_local::types_ollama::{ChatMessage, ToolCallFunction, ToolCallOllama};

fn assistant(path: &str) -> ChatMessage {
    ChatMessage::assistant(
        String::new(),
        None,
        Some(vec![ToolCallOllama {
            id: None,
            extra_content: None,
            function: ToolCallFunction {
                name: "read_file".to_string(),
                arguments: serde_json::json!({ "path": path }),
            },
        }]),
    )
}

fn tool(content: &str) -> ChatMessage {
    ChatMessage::tool(content.to_string(), None, None)
}

#[test]
fn keeps_three_recent_file_events() {
    let messages = vec![
        assistant("a.rs"),
        tool("a"),
        assistant("b.rs"),
        tool("b"),
        assistant("c.rs"),
        tool("c"),
        assistant("d.rs"),
        tool("d"),
    ];
    let msg = recent_file_context_message(&messages, 200_000).unwrap();
    assert!(!msg.content.contains("a.rs"));
    assert!(msg.content.contains("b.rs"));
    assert!(msg.content.contains("d.rs"));
}

#[test]
fn keeps_recent_non_file_tool_events() {
    let messages = vec![
        ChatMessage::tool("cargo test ok".to_string(), None, Some("bash".to_string())),
        ChatMessage::tool(
            "page summary".to_string(),
            None,
            Some("web_fetch".to_string()),
        ),
    ];
    let msg = recent_file_context_message(&messages, 200_000).unwrap();
    assert!(msg.content.contains("Recent tool context"));
    assert!(msg.content.contains("cargo test ok"));
    assert!(msg.content.contains("page summary"));
}
