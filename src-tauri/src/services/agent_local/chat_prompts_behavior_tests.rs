use super::chat_prompts::prepare_messages_with_tools;
use super::system_prompt_types::{PromptSelection, PromptSource, SystemPromptView};
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
fn custom_prompt_replaces_beaver_instructions_but_keeps_dynamic_context() {
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

    let instructions = SystemPromptView {
        content: "CUSTOM MODEL BEHAVIOR".into(),
        source: PromptSource::Custom,
        selection: PromptSelection::Custom,
        disabled: false,
        native_prompt_available: false,
    };
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
        &instructions,
    );

    let system = &messages[0].content;
    assert!(system.contains("CUSTOM MODEL BEHAVIOR"));
    assert!(!system.contains("You are an autonomous coding agent"));
    assert!(!system.contains("# Style"));
    assert!(!system.contains("# Using your tools"));
    assert!(!system.contains("<communication_during_work>"));
    assert!(system.contains("# Environment"));
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
fn custom_chat_prompt_replaces_static_chatbot_instructions() {
    let mut messages = vec![ChatMessage {
        role: "user".into(),
        content: "hello".into(),
        ..Default::default()
    }];
    let enabled = enabled_tool_names();

    let instructions = SystemPromptView {
        content: "CUSTOM CHAT BEHAVIOR".into(),
        source: PromptSource::Custom,
        selection: PromptSelection::Custom,
        disabled: false,
        native_prompt_available: false,
    };
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
        &instructions,
    );

    let system = &messages[0].content;
    assert!(system.contains("CUSTOM CHAT BEHAVIOR"));
    assert!(!system.contains("conversational assistant"));
    assert!(!system.contains("# Capabilities"));
    assert!(!system.contains("# Modes"));
    assert!(system.contains("# Environment"));
}

#[test]
fn empty_custom_prompt_keeps_only_dynamic_system_context() {
    let mut messages = vec![ChatMessage {
        role: "user".into(),
        content: "hello".into(),
        ..Default::default()
    }];
    let enabled = enabled_tool_names();
    let instructions = SystemPromptView {
        content: String::new(),
        source: PromptSource::Custom,
        selection: PromptSelection::Disabled,
        disabled: true,
        native_prompt_available: false,
    };

    prepare_messages_with_tools(
        &mut messages,
        Path::new("/tmp/project"),
        true,
        Some(Path::new("/tmp/project")),
        true,
        Some("AGENTS.md context".into()),
        &[("Test skill".into(), "Test description".into())],
        "qwen3-32b",
        "auto",
        "French",
        &enabled,
        &instructions,
    );

    let system = &messages[0].content;
    assert!(!system.contains("autonomous coding agent"));
    assert!(system.contains("# Environment"));
    assert!(system.contains("AGENTS.md context"));
    assert!(system.contains("Test skill"));
    assert!(system.contains("You MUST respond in French"));
}
