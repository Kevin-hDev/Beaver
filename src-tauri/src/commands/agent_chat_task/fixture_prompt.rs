use crate::services::agent_local::types_ollama::ChatMessage;

const SYNTHETIC_SYSTEM_PROMPT: &str =
    "You are a bounded Beaver reasoning fixture. Use only the provided fixture tools, answer the user's request, and preserve the supplied conversation history.";

pub(super) fn prepare(messages: &mut Vec<ChatMessage>) {
    messages.retain(|message| message.role != "system");
    messages.insert(0, ChatMessage::system(SYNTHETIC_SYSTEM_PROMPT.to_string()));
}

#[cfg(test)]
mod tests {
    use crate::services::agent_local::types_ollama::ChatMessage;

    #[test]
    fn synthetic_prompt_replaces_private_system_context_and_preserves_history() {
        let mut messages = vec![
            ChatMessage::system("PRIVATE AGENTS and memory".to_string()),
            ChatMessage::user("first turn".to_string()),
            ChatMessage::assistant("red quadrant".to_string(), None, None, None, None)
                .with_images(vec!["fixture-image".to_string()]),
            ChatMessage::tool(
                "written fixture value".to_string(),
                Some("call-1".to_string()),
                Some("fixture.write_note".to_string()),
            ),
        ];

        super::prepare(&mut messages);

        assert_eq!(messages[0].role, "system");
        assert!(messages[0].content.contains("fixture"));
        assert!(!messages[0].content.contains("PRIVATE"));
        assert_eq!(messages[1].content, "first turn");
        assert_eq!(
            messages[2].images.as_deref(),
            Some(["fixture-image".to_string()].as_slice())
        );
        assert_eq!(messages[3].tool_name.as_deref(), Some("fixture.write_note"));
    }
}
