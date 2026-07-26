use serde::Deserialize;
use std::path::PathBuf;

use crate::services::git::{
    action_error::GitActionError, branch_commit, branch_delete, branch_merge, remote,
    worktree_delete,
};

use super::git::registered_project_path;

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GitDeleteMode {
    Clean,
    Discard,
    Preserve,
}

#[tauri::command]
pub async fn get_git_remote_status(path: String) -> Result<remote::RemoteStatus, String> {
    let repo_path = registered_project_path(&path).await?;
    tokio::task::spawn_blocking(move || remote::status(&repo_path))
        .await
        .map_err(|_| "Erreur interne".to_string())?
}

#[tauri::command]
pub async fn commit_git_changes(
    path: String,
    commit_description: Option<String>,
) -> Result<(), GitActionError> {
    let repo_path = registered_project_path(&path)
        .await
        .map_err(|_| GitActionError::RepositoryUnavailable)?;
    tokio::task::spawn_blocking(move || branch_commit::commit_all(&repo_path, commit_description))
        .await
        .map_err(|_| GitActionError::InternalError)?
}

#[tauri::command]
pub async fn push_git_branch(
    path: String,
    expected_branch: String,
) -> Result<(), remote::PushError> {
    let repo_path = registered_project_path(&path)
        .await
        .map_err(|_| remote::PushError::InternalError)?;
    let github = tokio::task::spawn_blocking({
        let repo_path = repo_path.clone();
        move || remote::remote_requires_github_token(&repo_path)
    })
    .await
    .map_err(|_| remote::PushError::InternalError)?;
    let token = if github && crate::services::mcp_oauth::storage::has_tokens("github") {
        crate::services::mcp_oauth::storage::get_valid_token("github")
            .await
            .ok()
    } else {
        None
    };
    tokio::task::spawn_blocking(move || {
        remote::push_current(&repo_path, Some(&expected_branch), token)
    })
    .await
    .map_err(|_| remote::PushError::InternalError)?
}

#[tauri::command]
pub async fn preview_git_branch_merge(
    path: String,
    source_branch: String,
    expected_target: String,
) -> Result<branch_merge::BranchMergePreview, branch_merge::MergeError> {
    let repo_path = registered_project_path(&path)
        .await
        .map_err(|_| branch_merge::MergeError::InternalError)?;
    tokio::task::spawn_blocking(move || {
        branch_merge::preview(&repo_path, &source_branch, &expected_target)
    })
    .await
    .map_err(|_| branch_merge::MergeError::InternalError)?
}

#[tauri::command]
pub async fn merge_git_branch(
    path: String,
    source_branch: String,
    expected_target: String,
    commit_changes: bool,
    commit_description: Option<String>,
) -> Result<(), branch_merge::MergeError> {
    let repo_path = registered_project_path(&path)
        .await
        .map_err(|_| branch_merge::MergeError::InternalError)?;
    tokio::task::spawn_blocking(move || {
        branch_merge::merge_current(
            &repo_path,
            &source_branch,
            &expected_target,
            commit_changes,
            commit_description,
        )
    })
    .await
    .map_err(|_| branch_merge::MergeError::InternalError)?
}

#[tauri::command]
pub async fn preview_git_branch_deletion(
    path: String,
    branch_name: String,
) -> Result<branch_delete::BranchDeletePreview, GitActionError> {
    let repo_path = registered_project_path(&path)
        .await
        .map_err(|_| GitActionError::RepositoryUnavailable)?;
    tokio::task::spawn_blocking(move || branch_delete::preview(&repo_path, &branch_name))
        .await
        .map_err(|_| GitActionError::InternalError)?
}

#[tauri::command]
pub async fn delete_git_branch(
    path: String,
    branch_name: String,
    mode: GitDeleteMode,
    commit_description: Option<String>,
) -> Result<(), GitActionError> {
    let repo_path = registered_project_path(&path)
        .await
        .map_err(|_| GitActionError::RepositoryUnavailable)?;
    tokio::task::spawn_blocking(move || match mode {
        GitDeleteMode::Clean => branch_delete::delete_clean(&repo_path, &branch_name),
        GitDeleteMode::Discard => branch_delete::discard_and_delete(&repo_path, &branch_name),
        GitDeleteMode::Preserve => {
            branch_delete::preserve_and_delete(&repo_path, &branch_name, commit_description)
        }
    })
    .await
    .map_err(|_| GitActionError::InternalError)?
}

#[tauri::command]
pub async fn preview_git_worktree_deletion(
    path: String,
    worktree_path: String,
) -> Result<worktree_delete::WorktreeDeletePreview, GitActionError> {
    let repo_path = registered_project_path(&path)
        .await
        .map_err(|_| GitActionError::RepositoryUnavailable)?;
    worktree_delete::preview(&repo_path, &PathBuf::from(worktree_path)).await
}

#[tauri::command]
pub async fn delete_git_worktree(
    path: String,
    worktree_path: String,
    mode: GitDeleteMode,
    commit_description: Option<String>,
) -> Result<(), GitActionError> {
    let repo_path = registered_project_path(&path)
        .await
        .map_err(|_| GitActionError::RepositoryUnavailable)?;
    let worktree_path = PathBuf::from(worktree_path);
    match mode {
        GitDeleteMode::Clean => worktree_delete::remove_clean(&repo_path, &worktree_path).await,
        GitDeleteMode::Discard => {
            worktree_delete::discard_and_remove(&repo_path, &worktree_path).await
        }
        GitDeleteMode::Preserve => {
            worktree_delete::preserve_and_remove(&repo_path, &worktree_path, commit_description)
                .await
        }
    }
}
