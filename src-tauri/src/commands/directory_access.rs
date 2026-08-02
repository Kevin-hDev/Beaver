use crate::services::agent_local::directory_access::{self, DirectoryAccessDecision};
use crate::services::config as config_service;
use std::path::Path;

#[tauri::command]
pub async fn set_allowed_paths(
    app: tauri::AppHandle,
    paths: Vec<String>,
    streams: tauri::State<'_, crate::ActiveStreams>,
) -> Result<Vec<String>, String> {
    let normalized = directory_access::normalize_allowed_paths(paths)?;
    let roots = normalized
        .iter()
        .map(std::path::PathBuf::from)
        .collect::<Vec<_>>();
    let restricted = !directory_access::roots_allow_shell(&roots);
    let mut config = config_service::read_config()?;
    config.advanced.allowed_paths = normalized.clone();
    config_service::write_config(&config)?;
    if restricted {
        super::agent_chat_cancel::cancel_all_agent_requests(&app, &streams).await;
        crate::services::agent_local::tool_bash_registry::stop_all().await;
    }
    Ok(normalized)
}

#[tauri::command]
pub fn validate_session_directory_access(path: String) -> Result<DirectoryAccessDecision, String> {
    directory_access::decision(Path::new(&path))
}
