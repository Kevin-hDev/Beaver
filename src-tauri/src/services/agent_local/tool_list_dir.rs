use std::path::{Path, PathBuf};

use super::types_tools::ToolResult;

const MAX_LIST_ENTRIES: usize = 500;
const MAX_DEPTH: u32 = 3;

enum Work {
    ReadDirectory(PathBuf, u32),
    Entry(tokio::fs::DirEntry, u32),
}

pub async fn list_dir(path: &str, working_dir: &Path) -> ToolResult {
    let resolved = match super::tool_files::resolve_read_path(path, working_dir) {
        Ok(path) => path,
        Err(error) => {
            return super::tool_file_error::path_failure(
                error,
                "directory_not_found",
                "directory_read_denied",
                "invalid_directory_path",
            )
        }
    };
    let mut entries = Vec::new();
    let mut stack = vec![Work::ReadDirectory(resolved, 0)];
    let mut pending_entries = 0usize;
    let mut read_failures = 0usize;
    let mut metadata_failures = 0usize;
    let mut truncated = false;
    let mut visited_root = false;

    while let Some(work) = stack.pop() {
        match work {
            Work::ReadDirectory(dir, depth) => {
                if truncated {
                    continue;
                }
                let mut read_dir = match tokio::fs::read_dir(&dir).await {
                    Ok(read_dir) => read_dir,
                    Err(error) if !visited_root => {
                        return super::tool_file_error::directory_failure(error)
                    }
                    Err(_) => {
                        read_failures = read_failures.saturating_add(1);
                        continue;
                    }
                };
                visited_root = true;
                let remaining = MAX_LIST_ENTRIES
                    .saturating_sub(entries.len())
                    .saturating_sub(pending_entries);
                let mut children = Vec::new();
                loop {
                    match read_dir.next_entry().await {
                        Ok(Some(entry)) if visible(&entry) => {
                            if children.len() >= remaining {
                                truncated = true;
                                break;
                            }
                            children.push(entry);
                        }
                        Ok(Some(_)) => {}
                        Ok(None) => break,
                        Err(_) => {
                            read_failures = read_failures.saturating_add(1);
                            break;
                        }
                    }
                }
                children.sort_by_key(tokio::fs::DirEntry::file_name);
                pending_entries = pending_entries.saturating_add(children.len());
                for entry in children.into_iter().rev() {
                    stack.push(Work::Entry(entry, depth));
                }
            }
            Work::Entry(entry, depth) => {
                pending_entries = pending_entries.saturating_sub(1);
                let name = entry.file_name().to_string_lossy().to_string();
                let file_type = match entry.file_type().await {
                    Ok(file_type) => file_type,
                    Err(_) => {
                        metadata_failures = metadata_failures.saturating_add(1);
                        entries.push(format!("{}{name}", "  ".repeat(depth as usize)));
                        continue;
                    }
                };
                let is_dir = file_type.is_dir();
                entries.push(format!(
                    "{}{name}{}",
                    "  ".repeat(depth as usize),
                    if is_dir { "/" } else { "" }
                ));
                if is_dir && depth < MAX_DEPTH {
                    stack.push(Work::ReadDirectory(entry.path(), depth + 1));
                }
            }
        }
    }

    render(entries, read_failures, metadata_failures, truncated)
}

fn visible(entry: &tokio::fs::DirEntry) -> bool {
    let name = entry.file_name();
    let name = name.to_string_lossy();
    !name.starts_with('.') && name != "node_modules" && name != "target"
}

fn render(
    entries: Vec<String>,
    read_failures: usize,
    metadata_failures: usize,
    truncated: bool,
) -> ToolResult {
    let content = if entries.is_empty() {
        "(dossier vide)".to_string()
    } else {
        entries.join("\n")
    };
    let mut warnings = Vec::new();
    if read_failures > 0 {
        warnings.push(format!("{read_failures} dossier(s) n'ont pas pu être lus."));
    }
    if metadata_failures > 0 {
        warnings.push(format!("{metadata_failures} entrée(s) n'ont pas pu être identifiées."));
    }
    if truncated {
        warnings.push(format!("Liste limitée à {MAX_LIST_ENTRIES} entrées."));
    }
    let mut result = if warnings.is_empty() {
        ToolResult::ok(content)
    } else {
        ToolResult::partial(content, warnings)
    };
    result.mark_truncated(truncated);
    result
}
