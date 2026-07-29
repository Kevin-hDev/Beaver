use super::types_tools::ToolResult;
use std::path::{Path, PathBuf};

pub(super) fn append(mut result: ToolResult, working_dir: &Path) -> ToolResult {
    if result.is_error || result.affected_paths.is_empty() {
        return result;
    }
    let Some(root) = canonical(working_dir, working_dir) else {
        return result;
    };
    let allowed = allowed_roots(&root);
    let outside = result
        .affected_paths
        .iter()
        .take(super::tool_file_changes::MAX_FILE_CHANGES)
        .filter_map(|path| canonical(Path::new(path), &root))
        .any(|path| !allowed.iter().any(|allowed| path.starts_with(allowed)));
    if !outside {
        return result;
    }
    let Some(root) = safe_path(&root) else {
        return result;
    };
    result.content.push_str(
        "\n\n[WORKSPACE NOTICE: This operation changed files outside the active workspace. \
         Return to the active workspace unless the user explicitly requested another location. \
         Active workspace: ",
    );
    result.content.push_str(root);
    result.content.push(']');
    result
}

fn allowed_roots(working_dir: &Path) -> Vec<PathBuf> {
    let mut roots = vec![working_dir.to_path_buf()];
    let app_workspaces = crate::services::paths::data_dir().join("session-workspaces");
    if working_dir.starts_with(&app_workspaces) && working_dir.file_name().is_some_and(|v| v == "work")
    {
        if let Some(outputs) = working_dir.parent().map(|parent| parent.join("outputs")) {
            roots.push(outputs);
        }
    }
    if let Ok(config) = crate::services::config::read_config() {
        if let Some(outputs) = crate::models::config::normalize_optional_directory(
            &config.advanced.session_outputs_directory,
        )
        .filter(|value| !value.is_empty())
        {
            roots.push(PathBuf::from(outputs));
        }
    }
    roots
}

fn canonical(path: &Path, base: &Path) -> Option<PathBuf> {
    super::security::canonicalize_candidate(path, base).ok()
}

fn safe_path(path: &Path) -> Option<&str> {
    let value = path.to_str()?;
    (!value.is_empty() && value.len() <= 4_096 && !value.chars().any(char::is_control))
        .then_some(value)
}

#[cfg(test)]
#[path = "tool_workspace_notice_tests.rs"]
mod tests;
