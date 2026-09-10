use crate::services::agent_local::types_ollama::ChatMessage;

const MAX_TOTAL_RESULT_CHARS: usize = 100_000;
const MAX_OLD_RESULTS: usize = 4_096;
const MAX_RESULT_TREE_ENTRIES: usize = 100_000;
const MAX_RESULT_TREE_DEPTH: usize = 64;
const RESULT_MAX_AGE: std::time::Duration = std::time::Duration::from_secs(86_400);
const PERSIST_MARKER: &str = "[Résultat complet disponible : ";

pub const CLEARED_PLACEHOLDER: &str =
    "[Sortie précédente retirée du contexte pour respecter la limite.]";

pub struct RemovalOutcome {
    pub removed: usize,
    pub failed: Vec<(std::path::PathBuf, std::io::Error)>,
}

pub fn old_results_in(
    root: &std::path::Path,
    now: std::time::SystemTime,
) -> std::io::Result<Vec<(std::path::PathBuf, u64)>> {
    old_results_in_bounded(root, now, MAX_OLD_RESULTS)
}

pub(crate) fn old_results_in_bounded(
    root: &std::path::Path,
    now: std::time::SystemTime,
    max_results: usize,
) -> std::io::Result<Vec<(std::path::PathBuf, u64)>> {
    let directory = root.join("tool-results");
    let metadata = match std::fs::symlink_metadata(&directory) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(std::io::Error::from(std::io::ErrorKind::InvalidInput));
    }
    let cutoff = now
        .checked_sub(RESULT_MAX_AGE)
        .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
    let mut selected = Vec::new();
    for (index, entry) in std::fs::read_dir(&directory)?.enumerate() {
        if index == max_results {
            return Ok(selected);
        }
        let Ok(entry) = entry else { continue };
        let path = entry.path();
        let Ok(metadata) = std::fs::symlink_metadata(&path) else {
            continue;
        };
        let Ok(modified) = metadata.modified() else {
            continue;
        };
        if metadata.file_type().is_symlink() || modified >= cutoff {
            continue;
        }
        let mut visited = 0_usize;
        let Ok(bytes) = result_tree_size(&path, 0, &mut visited) else {
            continue;
        };
        selected.push((path, bytes));
    }
    Ok(selected)
}

fn result_tree_size(
    path: &std::path::Path,
    depth: usize,
    visited: &mut usize,
) -> std::io::Result<u64> {
    if depth >= MAX_RESULT_TREE_DEPTH || *visited >= MAX_RESULT_TREE_ENTRIES {
        return Err(std::io::Error::from(std::io::ErrorKind::InvalidData));
    }
    *visited += 1;
    let metadata = std::fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() {
        return Err(std::io::Error::from(std::io::ErrorKind::InvalidInput));
    }
    if metadata.is_file() {
        return Ok(metadata.len());
    }
    if !metadata.is_dir() {
        return Err(std::io::Error::from(std::io::ErrorKind::InvalidInput));
    }
    let mut bytes = 0_u64;
    for entry in std::fs::read_dir(path)? {
        bytes = bytes.saturating_add(result_tree_size(&entry?.path(), depth + 1, visited)?);
    }
    Ok(bytes)
}

pub fn remove_results(
    root: &std::path::Path,
    paths: &[std::path::PathBuf],
) -> RemovalOutcome {
    let mut outcome = RemovalOutcome {
        removed: 0,
        failed: Vec::with_capacity(paths.len()),
    };
    for path in paths {
        match remove_result(root, path) {
            Ok(()) => outcome.removed += 1,
            Err(error) => outcome.failed.push((path.clone(), error)),
        }
    }
    outcome
}

fn remove_result(root: &std::path::Path, path: &std::path::Path) -> std::io::Result<()> {
    let root_metadata = std::fs::symlink_metadata(root)?;
    let results = root.join("tool-results");
    let results_metadata = std::fs::symlink_metadata(&results)?;
    let parent = path
        .parent()
        .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::InvalidInput))?;
    let parent_metadata = std::fs::symlink_metadata(parent)?;
    let metadata = std::fs::symlink_metadata(path)?;
    if root_metadata.file_type().is_symlink()
        || !root_metadata.is_dir()
        || results_metadata.file_type().is_symlink()
        || !results_metadata.is_dir()
        || parent_metadata.file_type().is_symlink()
        || !parent_metadata.is_dir()
        || metadata.file_type().is_symlink()
    {
        return Err(std::io::Error::from(std::io::ErrorKind::InvalidInput));
    }
    if std::fs::canonicalize(parent)? != std::fs::canonicalize(results)? {
        return Err(std::io::Error::from(std::io::ErrorKind::InvalidInput));
    }
    if metadata.is_dir() {
        std::fs::remove_dir_all(path)
    } else if metadata.is_file() {
        std::fs::remove_file(path)
    } else {
        Err(std::io::Error::from(std::io::ErrorKind::InvalidInput))
    }
}

/// Supprime les dossiers de résultats persistés datant de plus de 24h.
pub fn cleanup_old_results() {
    let root = crate::services::paths::data_dir();
    let selected = match old_results_in(&root, std::time::SystemTime::now()) {
        Ok(selected) => selected,
        Err(_) => {
            ::log::error!("[tool-results] cleanup inventory unavailable");
            return;
        }
    };
    let paths = selected
        .into_iter()
        .map(|(path, _)| path)
        .collect::<Vec<_>>();
    let outcome = remove_results(&root, &paths);
    if !outcome.failed.is_empty() {
        ::log::error!(
            "[tool-results] cleanup failures count={}",
            outcome.failed.len()
        );
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
