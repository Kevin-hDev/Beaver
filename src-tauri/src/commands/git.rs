use crate::services::agent_local::project_store;
use crate::services::git::{
    action_error::GitActionError, branch, branch_commit, status, watcher, worktree_list,
};
use std::path::{Path, PathBuf};

pub(super) async fn registered_project_path(path: &str) -> Result<PathBuf, String> {
    let repo_path = Path::new(path);
    if !repo_path.is_dir() {
        return Err("Répertoire introuvable".to_string());
    }
    project_store::authorize_path(repo_path).await
}

#[tauri::command]
pub async fn start_git_watcher(app: tauri::AppHandle, path: String) -> Result<(), String> {
    let repo_path = registered_project_path(&path).await?;
    watcher::setup_git_watcher(app, repo_path)
}

#[tauri::command]
pub async fn stop_git_watcher(path: String) -> Result<(), String> {
    let repo_path = registered_project_path(&path).await?;
    watcher::stop_git_watcher(&repo_path)
}

#[tauri::command]
pub async fn list_git_branches(path: String) -> Result<Vec<branch::BranchInfo>, String> {
    let repo_path = registered_project_path(&path).await?;
    tokio::task::spawn_blocking(move || branch::list_branches(&repo_path))
        .await
        .map_err(|e| {
            eprintln!("[git] list_branches: {e}");
            "Erreur interne".to_string()
        })?
}

#[tauri::command]
pub async fn get_git_context(path: String) -> Result<branch::GitContext, String> {
    let repo_path = registered_project_path(&path).await?;
    tokio::task::spawn_blocking(move || branch::get_context(&repo_path))
        .await
        .map_err(|e| {
            eprintln!("[git] get_context: {e}");
            "Erreur interne".to_string()
        })
}

#[tauri::command]
pub async fn checkout_git_branch(path: String, branch_name: String) -> Result<(), GitActionError> {
    let repo_path = registered_project_path(&path)
        .await
        .map_err(|_| GitActionError::RepositoryUnavailable)?;
    tokio::task::spawn_blocking(move || branch::checkout_branch(&repo_path, &branch_name))
        .await
        .map_err(|e| {
            eprintln!("[git] checkout: {e}");
            GitActionError::InternalError
        })?
}

#[tauri::command]
pub async fn create_git_branch(
    path: String,
    branch_name: String,
) -> Result<(), branch::CreateBranchError> {
    let repo_path = registered_project_path(&path)
        .await
        .map_err(|_| branch::CreateBranchError::InternalError)?;
    tokio::task::spawn_blocking(move || branch::create_branch(&repo_path, &branch_name))
        .await
        .map_err(|e| {
            eprintln!("[git] create_branch: {e}");
            branch::CreateBranchError::InternalError
        })?
}

#[tauri::command]
pub async fn commit_and_checkout_git_branch(
    path: String,
    branch_name: String,
    commit_description: Option<String>,
) -> Result<(), GitActionError> {
    let repo_path = registered_project_path(&path)
        .await
        .map_err(|_| GitActionError::RepositoryUnavailable)?;
    tokio::task::spawn_blocking(move || {
        branch_commit::commit_all_and_checkout(&repo_path, &branch_name, commit_description)
    })
    .await
    .map_err(|e| {
        eprintln!("[git] commit_and_checkout: {e}");
        GitActionError::InternalError
    })?
}

#[tauri::command]
pub async fn list_git_dirty_files(path: String) -> Result<Vec<status::DirtyFile>, String> {
    let repo_path = registered_project_path(&path).await?;
    tokio::task::spawn_blocking(move || status::list_dirty_files(&repo_path))
        .await
        .map_err(|e| {
            eprintln!("[git] dirty_files: {e}");
            "Erreur interne".to_string()
        })?
}

#[tauri::command]
pub async fn list_git_worktrees(path: String) -> Result<Vec<worktree_list::WorktreeInfo>, String> {
    let repo_path = registered_project_path(&path).await?;
    worktree_list::list_worktrees(&repo_path).await
}
