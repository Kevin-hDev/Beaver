use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::process::Stdio;
use tokio::process::Command;
use tokio::time::{timeout, Duration};

pub(super) const GIT_REMOVE_TIMEOUT: Duration = Duration::from_secs(3);

pub(super) trait GitRemoveRunner: Sync {
    fn run<'a>(
        &'a self,
        path: &'a Path,
        retry_locked: bool,
    ) -> Pin<Box<dyn Future<Output = bool> + Send + 'a>>;
}

struct CommandGitRemoveRunner;

impl GitRemoveRunner for CommandGitRemoveRunner {
    fn run<'a>(
        &'a self,
        path: &'a Path,
        retry_locked: bool,
    ) -> Pin<Box<dyn Future<Output = bool> + Send + 'a>> {
        Box::pin(async move {
            let mut command = Command::new("git");
            command
                .arg("-C")
                .arg(path)
                .args(["worktree", "remove", "--force"]);
            if retry_locked {
                command.arg("--force");
            }
            command
                .arg(path)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .kill_on_drop(true)
                .status()
                .await
                .map(|status| status.success())
                .unwrap_or(false)
        })
    }
}

#[cfg(test)]
pub async fn remove(worktree_path: &str) -> Result<(), String> {
    let identity = super::subagent_worktree_identity::ManagedWorktreeIdentity::parse(worktree_path)?;
    remove_identity(identity).await
}

pub async fn remove_owned(
    worktree_path: &str,
    child_id: &str,
    execution_id: &str,
) -> Result<(), String> {
    let identity = super::subagent_worktree_identity::ManagedWorktreeIdentity::parse(worktree_path)?;
    identity.require_owner(child_id, execution_id)?;
    remove_identity(identity).await
}

pub async fn remove_for_child(worktree_path: &str, child_id: &str) -> Result<(), String> {
    let identity = super::subagent_worktree_identity::ManagedWorktreeIdentity::parse(worktree_path)?;
    identity.require_child(child_id)?;
    remove_identity(identity).await
}

async fn remove_identity(
    identity: super::subagent_worktree_identity::ManagedWorktreeIdentity,
) -> Result<(), String> {
    identity.reject_symlinks().await?;
    let canonical_root = match tokio::fs::canonicalize(&identity.root).await {
        Ok(root) => root,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(_) => return Err("Chemin worktree invalide".to_string()),
    };
    let child_dir = managed_child_dir(&identity.path, &canonical_root).await;
    let removal = remove_managed_path(&identity.path, &canonical_root).await;
    let parent_cleanup = cleanup_empty_parent(child_dir.as_deref()).await;

    removal.and(parent_cleanup)
}

async fn remove_managed_path(path: &Path, canonical_root: &Path) -> Result<(), String> {
    remove_managed_path_with_runner(path, canonical_root, &CommandGitRemoveRunner).await
}

pub(super) async fn remove_managed_path_with_runner(
    path: &Path,
    canonical_root: &Path,
    runner: &dyn GitRemoveRunner,
) -> Result<(), String> {
    let canonical_path = match tokio::fs::canonicalize(path).await {
        Ok(path) => path,
        Err(_) => match tokio::fs::symlink_metadata(path).await {
            Ok(_) => return Err("Chemin worktree invalide".to_string()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(_) => return Err("Chemin worktree invalide".to_string()),
        },
    };
    if canonical_path == canonical_root || !canonical_path.starts_with(canonical_root) {
        return Err("Chemin worktree hors du répertoire géré".to_string());
    }

    let first_removed = git_remove(runner, &canonical_path, false).await;
    match path_state(&canonical_path).await? {
        ManagedPathState::Missing => return Ok(()),
        ManagedPathState::Present if first_removed => {
            return Err("Suppression du worktree impossible".to_string());
        }
        ManagedPathState::Present => {}
    }
    let retry_removed = git_remove(runner, &canonical_path, true).await;
    match path_state(&canonical_path).await? {
        ManagedPathState::Missing => return Ok(()),
        ManagedPathState::Present if retry_removed => {
            return Err("Suppression du worktree impossible".to_string());
        }
        ManagedPathState::Present => {}
    }
    remove_partial_directory(&canonical_path).await?;
    verify_removed(&canonical_path).await
}

async fn git_remove(runner: &dyn GitRemoveRunner, path: &Path, retry_locked: bool) -> bool {
    matches!(
        timeout(GIT_REMOVE_TIMEOUT, runner.run(path, retry_locked)).await,
        Ok(true)
    )
}

async fn remove_partial_directory(path: &Path) -> Result<(), String> {
    if matches!(
        path_state(&path.join(".git")).await?,
        ManagedPathState::Present
    ) {
        return Err("Suppression du worktree impossible".to_string());
    }
    tokio::fs::remove_dir_all(path)
        .await
        .map_err(|_| "Suppression du worktree impossible".to_string())
}

async fn managed_child_dir(path: &Path, canonical_root: &Path) -> Option<PathBuf> {
    let parent = tokio::fs::canonicalize(path.parent()?).await.ok()?;
    if parent.parent()? != canonical_root {
        return None;
    }
    let child_id = parent.file_name()?.to_str()?;
    uuid::Uuid::parse_str(child_id).ok()?;
    Some(parent)
}

async fn cleanup_empty_parent(child_dir: Option<&Path>) -> Result<(), String> {
    let Some(child_dir) = child_dir else {
        return Ok(());
    };
    match tokio::fs::remove_dir(child_dir).await {
        Ok(()) => Ok(()),
        Err(error)
            if matches!(
                error.kind(),
                std::io::ErrorKind::NotFound | std::io::ErrorKind::DirectoryNotEmpty
            ) =>
        {
            Ok(())
        }
        Err(_) => Err("Nettoyage du répertoire worktree impossible".to_string()),
    }
}

async fn verify_removed(path: &Path) -> Result<(), String> {
    match path_state(path).await? {
        ManagedPathState::Missing => Ok(()),
        ManagedPathState::Present => Err("Suppression du worktree impossible".to_string()),
    }
}

enum ManagedPathState {
    Present,
    Missing,
}

async fn path_state(path: &Path) -> Result<ManagedPathState, String> {
    match tokio::fs::symlink_metadata(path).await {
        Ok(_) => Ok(ManagedPathState::Present),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Ok(ManagedPathState::Missing)
        }
        Err(_) => Err("Suppression du worktree impossible".to_string()),
    }
}
