use super::types_ollama::ChatMessage;

const MAX_CUSTOM_FOCUS_CHARS: usize = 2_000;

pub fn validate_custom_focus(focus: Option<String>) -> Result<Option<String>, String> {
    let Some(value) = focus else {
        return Ok(None);
    };
    if value.chars().count() > MAX_CUSTOM_FOCUS_CHARS {
        return Err("Action impossible".into());
    }
    Ok(Some(value))
}

pub fn build_summary_messages(serialized: &str, focus: Option<&str>) -> Vec<ChatMessage> {
    let focus_block = focus
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| format!("\n\n<focus>\n{value}\n</focus>"))
        .unwrap_or_default();
    vec![
        ChatMessage {
            role: "system".to_string(),
            content: system_prompt(),
            ..Default::default()
        },
        ChatMessage {
            role: "user".to_string(),
            content: format!(
                "<context>\nThe user cloned a coding session from an earlier message. Summarize only the conversation that happened after that clone point. The summary will be injected as hidden context in the new branch.\n</context>{focus_block}\n\n<conversation_after_clone>\n{serialized}\n</conversation_after_clone>"
            ),
            ..Default::default()
        },
    ]
}

fn system_prompt() -> String {
    [
        "<task>",
        "Create a concise exploration handoff for a cloned coding session.",
        "Do not invent file paths, commands, results, decisions, or errors.",
        "</task>",
        "<constraints>",
        "Preserve facts that help the clone continue work without reading the full parent session.",
        "Mention uncertainty explicitly when information is incomplete.",
        "Ignore unrelated chatter and repeated tool noise.",
        "</constraints>",
        "<output_format>",
        "Use these exact headings:",
        "Goal:",
        "Constraints:",
        "Progress:",
        "  Done:",
        "  In Progress:",
        "  Blocked:",
        "Decisions:",
        "Files Read:",
        "Files Modified:",
        "Errors or Risks:",
        "Next Steps:",
        "</output_format>",
        "<success_criteria>",
        "The clone can continue from the selected message with the useful context that happened later in the parent session.",
        "</success_criteria>",
    ]
    .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_oversized_custom_focus() {
        let focus = "a".repeat(MAX_CUSTOM_FOCUS_CHARS + 1);
        assert!(validate_custom_focus(Some(focus)).is_err());
    }

    #[test]
    fn prompt_uses_structured_handoff() {
        let messages = build_summary_messages("conversation", Some("focus"));
        assert!(messages[0].content.contains("Goal:"));
        assert!(messages[0].content.contains("Next Steps:"));
        assert!(messages[1].content.contains("<focus>"));
    }
}
