use super::types_ollama::ChatMessage;

pub fn append(messages: &mut [ChatMessage], enabled_tool_names: &[String]) {
    if !enabled_tool_names
        .iter()
        .any(|name| name == crate::services::extensions::SEARCH_TOOL_NAME)
    {
        return;
    }
    let Some(system) = messages.first_mut().filter(|message| message.role == "system") else {
        return;
    };
    system.content.push_str(
        "\n\nEnabled extensions may provide additional capabilities. Use the extension tools \
         supplied for the task; before installing dependencies or recreating a capability with \
         Bash, use search_extension_tools to discover another enabled extension.",
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_mentions_discovery_when_the_tool_is_available() {
        let mut messages = vec![ChatMessage {
            role: "system".to_string(),
            content: "Base".to_string(),
            ..Default::default()
        }];

        append(&mut messages, &[]);
        assert_eq!(messages[0].content, "Base");

        append(&mut messages, &["search_extension_tools".to_string()]);
        assert!(messages[0].content.contains("search_extension_tools"));
    }
}
