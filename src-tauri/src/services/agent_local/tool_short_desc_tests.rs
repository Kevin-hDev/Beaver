#![cfg(test)]

use super::tool_catalog::catalog;
use super::tool_short_desc::tool_short_desc;
use super::types_ollama::ChatMessage;
use std::path::Path;

/// Helper: build a fresh system message vector and run prompt assembly in the
/// given mode. Returns the assembled system prompt content.
fn assembled_system_prompt(mode: &str) -> String {
    let mut messages: Vec<ChatMessage> = vec![];
    super::chat_prompts::prepare_messages(
        &mut messages,
        Path::new("."),
        false,
        None,
        true,
        None,
        &[],
        "qwen3",
        mode,
        "",
    );
    messages
        .first()
        .expect("system prompt should be present")
        .content
        .clone()
}

/// Every optional tool in the catalog must have a short description so the
/// disabled-tools hint is always complete.
#[test]
fn every_optional_tool_has_a_short_description() {
    let missing: Vec<&str> = catalog()
        .iter()
        .filter(|t| !t.locked)
        .filter(|t| tool_short_desc(t.id).is_none())
        .map(|t| t.id)
        .collect();
    assert!(
        missing.is_empty(),
        "optional tools without a short description: {missing:?}"
    );
}

/// Locked tools are never part of the hint, so they must return `None`.
#[test]
fn locked_tools_return_none() {
    for tool in catalog().iter().filter(|t| t.locked) {
        assert!(
            tool_short_desc(tool.id).is_none(),
            "locked tool {} should not have a short description",
            tool.id
        );
    }
}

#[test]
fn unknown_id_returns_none() {
    assert!(tool_short_desc("does_not_exist").is_none());
    assert!(tool_short_desc("").is_none());
}

/// Keep hints short — long descriptions would bloat the system prompt.
#[test]
fn descriptions_are_under_100_chars() {
    for tool in catalog().iter().filter(|t| !t.locked) {
        if let Some(desc) = tool_short_desc(tool.id) {
            assert!(
                desc.len() <= 100,
                "{} description is too long ({} chars): {desc}",
                tool.id,
                desc.len()
            );
        }
    }
}

/// In the default test fixture every optional tool is enabled, so the hint
/// section must NOT be present in the assembled agent prompt.
#[test]
fn hint_absent_when_all_optional_tools_enabled() {
    let system = assembled_system_prompt("agent");
    assert!(
        !system.contains("## Disabled tools"),
        "hint should not be injected when all optional tools are enabled"
    );
}

/// When some optional tools are disabled, the hint MUST be present and MUST
/// list every disabled tool id with its short description.
#[test]
fn hint_present_when_some_tools_disabled() {
    use super::chat_prompts::prepare_messages_with_tools;

    // Enable only the locked tools (no optional tool), so every optional tool
    // is treated as disabled for this test.
    let enabled: Vec<String> = catalog()
        .iter()
        .filter(|t| t.locked)
        .map(|t| t.id.to_string())
        .collect();

    let mut messages: Vec<ChatMessage> = vec![];
    prepare_messages_with_tools(
        &mut messages,
        Path::new("."),
        false,
        None,
        true,
        None,
        &[],
        "qwen3",
        "agent",
        "",
        &enabled,
        None,
    );
    let system = messages
        .first()
        .expect("system prompt present")
        .content
        .clone();

    assert!(
        system.contains("## Disabled tools"),
        "hint section should be injected when optional tools are disabled"
    );
    // A few known optional tools must appear with their short description.
    assert!(
        system.contains("- forecast:"),
        "hint should list the forecast tool"
    );
    assert!(
        system.contains("- todo_write:"),
        "hint should list the todo_write tool"
    );
}

/// Mode chat must not receive the hint (it has its own tool set and never
/// calls prepend_disabled_tools_hint).
#[test]
fn hint_absent_in_chat_mode() {
    let system = assembled_system_prompt("chat");
    assert!(
        !system.contains("## Disabled tools"),
        "hint should not be injected in chat mode"
    );
}
