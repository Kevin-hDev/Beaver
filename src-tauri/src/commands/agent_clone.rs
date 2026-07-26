use crate::services::agent_local::clone_git::CloneGitBranchResult;
use crate::services::agent_local::clone_session::CloneSessionResult;
use crate::services::agent_local::session_tabs::SessionTabs;
use crate::services::agent_local::types_session::CloneMode;
use crate::services::git::action_error::GitActionError;
use crate::services::git::branch::CreateBranchError;
use std::path::{Path, PathBuf};

async fn registered_project_path(path: &str) -> Result<PathBuf, String> {
    let repo_path = Path::new(path);
    if !repo_path.is_dir() {
        return Err("Répertoire introuvable".into());
    }
    crate::services::agent_local::project_store::authorize_path(repo_path).await
}

#[tauri::command]
pub async fn clone_agent_session(
    session_id: String,
    message_id: String,
    mode: CloneMode,
    custom_focus: Option<String>,
    operation_id: Option<String>,
) -> Result<CloneSessionResult, String> {
    crate::services::agent_local::clone_session::clone_session(
        &session_id,
        &message_id,
        mode,
        custom_focus,
        operation_id,
    )
    .await
}

#[tauri::command]
pub async fn cancel_clone_summary(operation_id: String) -> Result<(), String> {
    crate::services::agent_local::clone_session::cancel_clone_summary(&operation_id).await
}

#[tauri::command]
pub async fn list_session_tabs(session_id: String) -> Result<SessionTabs, String> {
    crate::services::agent_local::session_tabs::list(&session_id).await
}

#[tauri::command]
pub async fn save_session_tabs(
    session_id: String,
    tabs: SessionTabs,
) -> Result<SessionTabs, String> {
    crate::services::agent_local::session_tabs::save_tabs(&session_id, tabs).await
}

#[tauri::command]
pub async fn close_session_tab(session_id: String, tab_id: String) -> Result<SessionTabs, String> {
    crate::services::agent_local::session_tabs::close_tab(&session_id, &tab_id).await
}

#[tauri::command]
pub async fn rename_session_tab(
    session_id: String,
    tab_id: String,
    label: String,
) -> Result<SessionTabs, String> {
    crate::services::agent_local::session_tabs::rename_tab(&session_id, &tab_id, &label).await
}

#[tauri::command]
pub async fn create_clone_git_branch(
    session_id: String,
    clone_session_id: String,
    path: String,
) -> Result<CloneGitBranchResult, CreateBranchError> {
    let repo_path = registered_project_path(&path)
        .await
        .map_err(|_| CreateBranchError::InternalError)?;
    crate::services::agent_local::clone_git::create_linked_branch(
        &session_id,
        &clone_session_id,
        &repo_path,
    )
    .await
}

#[tauri::command]
pub async fn unlink_clone_git_branch(
    session_id: String,
    clone_session_id: String,
) -> Result<SessionTabs, GitActionError> {
    crate::services::agent_local::clone_git::unlink_branch(&session_id, &clone_session_id).await
}

#[tauri::command]
pub async fn link_clone_git_branch(
    session_id: String,
    clone_session_id: String,
    path: String,
    branch_name: String,
) -> Result<SessionTabs, GitActionError> {
    let repo_path = registered_project_path(&path)
        .await
        .map_err(|_| GitActionError::RepositoryUnavailable)?;
    crate::services::agent_local::clone_git_link::link_existing_branch(
        &session_id,
        &clone_session_id,
        &repo_path,
        &branch_name,
    )
    .await
}

#[tauri::command]
pub async fn close_session_tab_and_cleanup_git_branch(
    session_id: String,
    tab_id: String,
    path: String,
    fallback_branch: Option<String>,
) -> Result<SessionTabs, GitActionError> {
    let repo_path = registered_project_path(&path)
        .await
        .map_err(|_| GitActionError::RepositoryUnavailable)?;
    crate::services::agent_local::clone_git::close_tab_with_branch_cleanup(
        &session_id,
        &tab_id,
        &repo_path,
        fallback_branch.as_deref(),
    )
    .await
}
