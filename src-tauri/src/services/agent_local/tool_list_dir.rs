use std::path::Path;

use super::types_tools::ToolResult;

const MAX_LIST_ENTRIES: usize = 500;
const MAX_DEPTH: u32 = 3;

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
    let mut stack = vec![(resolved, 0u32)];
    let mut read_failures = 0usize;
    let mut metadata_failures = 0usize;
    let mut truncated = false;
    let mut visited_root = false;

    while let Some((dir, depth)) = stack.pop() {
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
        let mut children = Vec::new();
        let remaining = MAX_LIST_ENTRIES.saturating_sub(entries.len());
        loop {
            match read_dir.next_entry().await {
                Ok(Some(entry)) => {
                    let name = entry.file_name();
                    let name = name.to_string_lossy();
                    if !name.starts_with('.') && name != "node_modules" && name != "target" {
                        if children.len() >= remaining {
                            truncated = true;
                            break;
                        }
                        children.push(entry);
                    }
                }
                Ok(None) => break,
                Err(_) => {
                    read_failures = read_failures.saturating_add(1);
                    break;
                }
            }
        }
        children.sort_by_key(tokio::fs::DirEntry::file_name);
        for entry in children {
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
                stack.push((entry.path(), depth + 1));
            }
        }
        if truncated {
            break;
        }
    }

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
