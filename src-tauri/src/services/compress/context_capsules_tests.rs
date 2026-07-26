use super::context_capsules::recent_file_context_message;
use crate::services::agent_local::types_ollama::{ChatMessage, ToolCallFunction, ToolCallOllama};

fn assistant(path: &str) -> ChatMessage {
    ChatMessage {
        role: "assistant".to_string(),
        content: String::new(),
        tool_calls: Some(vec![ToolCallOllama {
            id: None,
            extra_content: None,
            function: ToolCallFunction {
                name: "read_file".to_string(),
                arguments: serde_json::json!({ "path": path }),
            },
        }]),
        ..Default::default()
    }
}

fn tool(content: &str) -> ChatMessage {
    ChatMessage {
        role: "tool".to_string(),
        content: content.to_string(),
        ..Default::default()
    }
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
        ChatMessage {
            role: "tool".to_string(),
            tool_name: Some("bash".to_string()),
            content: "cargo test ok".to_string(),
            ..Default::default()
        },
        ChatMessage {
            role: "tool".to_string(),
            tool_name: Some("web_fetch".to_string()),
            content: "page summary".to_string(),
            ..Default::default()
        },
    ];
    let msg = recent_file_context_message(&messages, 200_000).unwrap();
    assert!(msg.content.contains("Recent tool context"));
    assert!(msg.content.contains("cargo test ok"));
    assert!(msg.content.contains("page summary"));
}
