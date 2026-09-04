use super::types_ollama::ChatMessage;

pub fn append(messages: &mut [ChatMessage], enabled_tool_names: &[String]) {
    if !enabled_tool_names.iter().any(|name| {
        name == crate::services::extensions::LIST_EXTENSIONS_TOOL_NAME
    }) || !enabled_tool_names.iter().any(|name| {
        name == crate::services::extensions::INSPECT_EXTENSIONS_TOOL_NAME
    })
    {
        return;
    }
    let Some(system) = messages
        .first_mut()
        .filter(|message| message.role == "system")
    else {
        return;
    };
    system.content.push_str(
        "\n\nEnabled extensions may provide additional capabilities. Use the extension tools \
         supplied for the task; before installing dependencies or recreating a capability with \
         Bash, the compact extension catalogue already provides known exact IDs: use \
         inspect_extensions directly with 1–4 known exact IDs. Use list_extensions only for the \
         complete view, descriptions, or counts, then inspect_extensions with 1–4 exact IDs. \
         Never use lexical or keyword search for extensions.",
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explains_direct_known_id_inspection_and_optional_full_listing() {
        let mut messages = vec![ChatMessage::system("Base".to_string())];

        append(&mut messages, &[]);
        assert_eq!(messages[0].content, "Base");

        append(&mut messages, &["list_extensions".to_string()]);
        assert_eq!(messages[0].content, "Base");

        append(
            &mut messages,
            &["list_extensions".to_string(), "inspect_extensions".to_string()],
        );
        assert!(messages[0]
            .content
            .contains("inspect_extensions directly with 1–4 known exact IDs"));
        assert!(messages[0]
            .content
            .contains("list_extensions only for the complete view, descriptions, or counts"));
    }
}
