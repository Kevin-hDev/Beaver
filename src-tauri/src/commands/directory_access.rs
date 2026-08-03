use crate::services::agent_local::directory_access::{self, DirectoryAccessDecision};
use std::path::Path;

#[tauri::command]
pub async fn set_allowed_paths(
    app: tauri::AppHandle,
    paths: Vec<String>,
    streams: tauri::State<'_, crate::ActiveStreams>,
) -> Result<Vec<String>, String> {
    let normalized = directory_access::normalize_allowed_paths(paths)?;
    let new_roots = normalized
        .iter()
        .map(std::path::PathBuf::from)
        .collect::<Vec<_>>();
    let old_roots = directory_access::configured_roots().ok();
    let access_narrowed = old_roots
        .as_deref()
        .is_none_or(|old| access_is_narrower(old, &new_roots));
    directory_access::replace_policy(normalized.clone())?;
    if access_narrowed {
        super::agent_chat_cancel::cancel_all_agent_requests(&app, &streams).await;
        crate::services::agent_local::tool_bash_registry::stop_all().await;
    }
    Ok(normalized)
}

fn access_is_narrower(old_roots: &[std::path::PathBuf], new_roots: &[std::path::PathBuf]) -> bool {
    if directory_access::roots_allow_full_disk(new_roots) {
        return false;
    }
    old_roots
        .iter()
        .any(|old| !new_roots.iter().any(|new| old.starts_with(new)))
}

#[tauri::command]
pub fn validate_session_directory_access(path: String) -> Result<DirectoryAccessDecision, String> {
    directory_access::decision(Path::new(&path))
}

#[cfg(test)]
#[path = "directory_access_tests.rs"]
mod tests;
