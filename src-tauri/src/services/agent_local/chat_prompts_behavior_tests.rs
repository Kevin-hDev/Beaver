use super::chat_prompts::prepare_messages_with_tools;
use super::tool_catalog;
use super::types_ollama::ChatMessage;
use std::path::Path;

fn enabled_tool_names() -> Vec<String> {
    tool_catalog::catalog()
        .iter()
        .map(|tool| tool.id.to_string())
        .collect()
}

#[test]
fn custom_behavior_replaces_defaults_but_keeps_tools_and_context_files() {
    let mut messages = vec![ChatMessage {
        role: "user".into(),
        content: "hello".into(),
        ..Default::default()
    }];
    let context = [
        "AGENTS.md context",
        "identity.md context",
        "principles.md context",
        "User.md context",
        "idea-discovery.md context",
    ]
    .join("\n");
    let enabled = enabled_tool_names();

    prepare_messages_with_tools(
        &mut messages,
        Path::new("/tmp/project"),
        false,
        None,
        true,
        Some(context),
        &[("Test skill".into(), "Test description".into())],
        "qwen3-32b",
        "auto",
        "French",
        &enabled,
        Some("CUSTOM MODEL BEHAVIOR"),
    );

    let system = &messages[0].content;
    assert!(system.contains("CUSTOM MODEL BEHAVIOR"));
    assert!(!system.contains("You are an autonomous coding agent"));
    assert!(!system.contains("# Style"));
    assert!(system.contains("# Using your tools"));
    assert!(system.contains("<communication_during_work>"));
    assert!(system.contains("Test skill"));
    assert!(system.contains("You MUST respond in French"));
    for file_context in [
        "AGENTS.md context",
        "identity.md context",
        "principles.md context",
        "User.md context",
        "idea-discovery.md context",
    ] {
        assert!(system.contains(file_context));
    }
}

#[test]
fn custom_chat_behavior_keeps_chat_tools_and_mode_boundaries() {
    let mut messages = vec![ChatMessage {
        role: "user".into(),
        content: "hello".into(),
        ..Default::default()
    }];
    let enabled = enabled_tool_names();

    prepare_messages_with_tools(
        &mut messages,
        Path::new("/tmp/project"),
        false,
        None,
        true,
        None,
        &[],
        "gemma-4-e4b",
        "chat",
        "",
        &enabled,
        Some("CUSTOM CHAT BEHAVIOR"),
    );

    let system = &messages[0].content;
    assert!(system.contains("CUSTOM CHAT BEHAVIOR"));
    assert!(!system.contains("conversational assistant"));
    assert!(system.contains("web_search"));
    assert!(system.contains("Chatbot"));
}
