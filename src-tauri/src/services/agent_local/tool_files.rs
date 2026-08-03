use crate::services::agent_local::security;
use crate::services::agent_local::types_tools::ToolResult;
use crate::services::agent_local::tool_result_contract::ToolErrorCategory;
use std::path::{Path, PathBuf};
use super::tool_file_error::io_failure;

pub use super::tool_file_write::write_file;
#[cfg(test)]
pub(crate) use super::tool_file_write::write_file_in_roots;

const MAX_READ_SIZE: u64 = 20 * 1024 * 1024;
pub const DEFAULT_LIMIT: usize = 2000;
const MAX_LIMIT: usize = 50_000;

pub(crate) fn resolve_read_path(path: &str, working_dir: &Path) -> Result<PathBuf, String> {
    let p = Path::new(path);
    let raw = if p.is_absolute() {
        p.to_path_buf()
    } else {
        working_dir.join(p)
    };
    security::validate_read_path(&raw, working_dir)
}

fn resolve_write_path(path: &str, working_dir: &Path) -> Result<PathBuf, String> {
    let p = Path::new(path);
    let raw = if p.is_absolute() {
        p.to_path_buf()
    } else {
        working_dir.join(p)
    };
    security::validate_write_path(&raw, working_dir)
}

pub async fn read_file(path: &str, working_dir: &Path, offset: usize, limit: usize) -> ToolResult {
    let resolved = match resolve_read_path(path, working_dir) {
        Ok(p) => p,
        Err(error) => {
            return super::tool_file_error::path_failure(
                error,
                "file_not_found",
                "file_read_denied",
                "invalid_file_path",
            )
        }
    };
    match tokio::fs::metadata(&resolved).await {
        Ok(meta) if meta.len() > MAX_READ_SIZE => {
            return ToolResult::error(
                "Fichier trop volumineux (max 20MB)",
                "file_too_large",
                ToolErrorCategory::Validation,
                false,
            );
        }
        Err(e) => return io_failure(e, "file_metadata_failed"),
        _ => {}
    }
    let raw = match tokio::fs::read_to_string(&resolved).await {
        Ok(c) => c,
        Err(e) => return io_failure(e, "file_read_failed"),
    };
    let lines: Vec<&str> = raw.lines().collect();
    let total = lines.len();
    let start = offset.min(total);
    let limit = limit.min(MAX_LIMIT);
    let end = start.saturating_add(limit).min(total);
    let slice = &lines[start..end];
    let mut output = String::with_capacity(slice.len() * 80);
    for (i, line) in slice.iter().enumerate() {
        let line_num = start + i + 1;
        output.push_str(&format!("{line_num}\t{line}\n"));
    }
    let remaining = total.saturating_sub(end);
    if remaining > 0 {
        output.push_str(&format!(
            "\n[{remaining} ligne(s) restante(s) — utilise offset={end} pour la suite]"
        ));
    }
    let mut result = ToolResult::ok(output);
    result.mark_truncated(remaining > 0);
    result
}

pub async fn edit_file(
    path: &str,
    old_string: &str,
    new_string: &str,
    working_dir: &Path,
) -> ToolResult {
    let resolved = match resolve_write_path(path, working_dir) {
        Ok(p) => p,
        Err(error) => {
            return super::tool_file_error::path_failure(
                error,
                "file_not_found",
                "write_path_denied",
                "invalid_write_path",
            )
        }
    };
    let content = match tokio::fs::read_to_string(&resolved).await {
        Ok(c) => c,
        Err(e) => return io_failure(e, "file_read_failed"),
    };
    let count = content.matches(old_string).count();
    if count == 0 {
        return ToolResult::error(
            "Chaîne non trouvée",
            "edit_match_not_found",
            ToolErrorCategory::NotFound,
            false,
        );
    }
    if count > 1 {
        return ToolResult::error(
            format!("Chaîne trouvée {count} fois (doit être unique)"),
            "edit_match_ambiguous",
            ToolErrorCategory::Conflict,
            false,
        )
        .with_error_hint(
            "Ajouter des lignes avant ou après dans old_string pour rendre la correspondance unique.",
        );
    }
    let start_line = content[..content.find(old_string).unwrap_or(0)]
        .chars()
        .filter(|c| *c == '\n')
        .count()
        + 1;
    let updated = content.replacen(old_string, new_string, 1);
    match tokio::fs::write(&resolved, &updated).await {
        Ok(()) => ToolResult::ok(format!(
            "Modifié: {} (ligne {})",
            resolved.display(),
            start_line
        ))
        .with_start_line(start_line),
        Err(e) => io_failure(e, "file_write_failed"),
    }
}

pub async fn list_dir(path: &str, working_dir: &Path) -> ToolResult {
    super::tool_list_dir::list_dir(path, working_dir).await
}
