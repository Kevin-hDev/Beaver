use crate::services::agent_local::types_ollama::ChatMessage;

const MAX_TOTAL_RESULT_CHARS: usize = 100_000;
const PERSIST_MARKER: &str = "[Résultat complet disponible : ";

pub const CLEARED_PLACEHOLDER: &str =
    "[Sortie précédente retirée du contexte pour respecter la limite.]";

/// Supprime les dossiers de résultats persistés datant de plus de 24h.
pub fn cleanup_old_results() {
    let persist_dir = crate::services::paths::data_dir().join("tool-results");
    if let Ok(entries) = std::fs::read_dir(&persist_dir) {
        let cutoff = std::time::SystemTime::now()
            .checked_sub(std::time::Duration::from_secs(86400))
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        for entry in entries.flatten() {
            if entry
                .metadata()
                .and_then(|metadata| metadata.modified())
                .is_ok_and(|modified| modified < cutoff)
            {
                let _ = std::fs::remove_dir_all(entry.path());
            }
        }
    }
}

pub fn apply_budget(messages: &mut [ChatMessage]) {
    let tool_indices: Vec<usize> = messages
        .iter()
        .enumerate()
        .filter(|(_, message)| {
            message.role == "tool"
                && !super::tool_result_model_compact::output_starts_with(
                    &message.content,
                    CLEARED_PLACEHOLDER,
                )
        })
        .map(|(index, _)| index)
        .collect();
    let total = tool_indices
        .iter()
        .map(|index| messages[*index].content.chars().count())
        .sum::<usize>();
    if total <= MAX_TOTAL_RESULT_CHARS {
        return;
    }

    let preserve_from = tool_indices.len().saturating_sub(2);
    let mut remaining = total;
    for index in &tool_indices[..preserve_from] {
        if remaining <= MAX_TOTAL_RESULT_CHARS {
            break;
        }
        let previous_chars = messages[*index].content.chars().count();
        let replacement = compacted_output(&messages[*index].content);
        let compacted = super::tool_result_model_compact::replace_output(
            &messages[*index].content,
            &replacement,
        );
        remaining = remaining
            .saturating_sub(previous_chars)
            .saturating_add(compacted.chars().count());
        messages[*index].content = compacted;
    }
}

fn compacted_output(content: &str) -> String {
    match extract_persist_path(content) {
        Some(path) => format!(
            "{CLEARED_PLACEHOLDER} Résultat complet lisible avec read_file : {path}"
        ),
        None => CLEARED_PLACEHOLDER.to_string(),
    }
}

fn extract_persist_path(content: &str) -> Option<&str> {
    let start = content.find(PERSIST_MARKER)? + PERSIST_MARKER.len();
    let end = content[start..].find(']')? + start;
    Some(&content[start..end])
}
