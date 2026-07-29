use crate::services::agent_local::types_ollama::ChatMessage;
use std::path::Path;

pub(super) fn append_outputs_directory(messages: &mut [ChatMessage], outputs_dir: Option<&Path>) {
    let Some(path) = outputs_dir.and_then(safe_prompt_path) else {
        return;
    };
    if let Some(first) = messages
        .first_mut()
        .filter(|message| message.role == "system")
    {
        first.content.push_str(
            "\n\n## Session workspace\n\
             - Put final deliverables requested by the user in the outputs directory.\n\
             - Outputs directory: ",
        );
        first.content.push_str(path);
    }
}

fn safe_prompt_path(path: &Path) -> Option<&str> {
    let value = path.to_str()?;
    (!value.is_empty() && value.len() <= 4_096 && !value.chars().any(char::is_control))
        .then_some(value)
}

#[cfg(test)]
#[path = "workspace_prompt_tests.rs"]
mod tests;
