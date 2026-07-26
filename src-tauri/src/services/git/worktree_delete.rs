use serde::Serialize;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;
use tokio::time::{timeout, Duration};

use super::{action_error::GitActionError, branch_commit, status, worktree_list};

const MAX_PATH_CHARS: usize = 4_096;
const REMOVE_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Clone, Serialize)]
pub struct WorktreeDeletePreview {
    pub path: String,
    pub branch: String,
    pub dirty_files: Vec<status::DirtyFile>,
}

pub async fn preview(
    repo_path: &Path,
    target_path: &Path,
) -> Result<WorktreeDeletePreview, GitActionError> {
    let (target, info) = validate_target(repo_path, target_path).await?;
    let dirty_target = target.clone();
    let dirty_files = tokio::task::spawn_blocking(move || status::list_dirty_files(&dirty_target))
        .await
        .map_err(|_| GitActionError::InternalError)?
        .map_err(|_| GitActionError::InternalError)?;
    Ok(WorktreeDeletePreview {
        path: target.to_string_lossy().to_string(),
        branch: info.branch,
        dirty_files,
    })
}

pub async fn remove_clean(repo_path: &Path, target_path: &Path) -> Result<(), GitActionError> {
    let (target, _) = validate_target(repo_path, target_path).await?;
    let dirty_target = target.clone();
    let dirty_count = tokio::task::spawn_blocking(move || {
        status::list_dirty_files(&dirty_target).map(|files| files.len())
    })
    .await
    .map_err(|_| GitActionError::InternalError)?
    .map_err(|_| GitActionError::InternalError)?;
    if dirty_count > 0 {
        return Err(GitActionError::DirtyWorktree { dirty_count });
    }
    remove(repo_path, &target, false).await
}

pub async fn discard_and_remove(
    repo_path: &Path,
    target_path: &Path,
) -> Result<(), GitActionError> {
    let (target, _) = validate_target(repo_path, target_path).await?;
    remove(repo_path, &target, true).await
}

pub async fn preserve_and_remove(
    repo_path: &Path,
    target_path: &Path,
    description: Option<String>,
) -> Result<(), GitActionError> {
    let (target, _) = validate_target(repo_path, target_path).await?;
    let commit_target = target.clone();
    tokio::task::spawn_blocking(move || branch_commit::commit_all(&commit_target, description))
        .await
        .map_err(|_| GitActionError::InternalError)??;
    remove(repo_path, &target, false).await
}

async fn validate_target(
    repo_path: &Path,
    target_path: &Path,
) -> Result<(PathBuf, worktree_list::WorktreeInfo), GitActionError> {
    if target_path.as_os_str().is_empty()
        || target_path.to_string_lossy().chars().count() > MAX_PATH_CHARS
    {
        return Err(GitActionError::WorktreeUnavailable);
    }
    let target =
        std::fs::canonicalize(target_path).map_err(|_| GitActionError::WorktreeUnavailable)?;
    let worktrees = worktree_list::list_worktrees(repo_path)
        .await
        .map_err(|_| GitActionError::InternalError)?;
    let info = worktrees
        .into_iter()
        .find(|worktree| {
            !worktree.is_current
                && std::fs::canonicalize(&worktree.path).ok().as_ref() == Some(&target)
        })
        .ok_or(GitActionError::WorktreeUnavailable)?;
    Ok((target, info))
}

async fn remove(repo_path: &Path, target_path: &Path, force: bool) -> Result<(), GitActionError> {
    let mut command = Command::new("git");
    command
        .arg("-C")
        .arg(repo_path)
        .args(["worktree", "remove"]);
    if force {
        command.args(["--force", "--force"]);
    }
    command
        .arg(target_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true);
    let result = timeout(REMOVE_TIMEOUT, command.status())
        .await
        .map_err(|_| GitActionError::DeleteFailed)?
        .map_err(|_| GitActionError::DeleteFailed)?;
    if !result.success() {
        return Err(GitActionError::DeleteFailed);
    }
    Ok(())
}
