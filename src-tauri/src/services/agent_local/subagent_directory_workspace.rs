use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;

pub async fn is_git_repository(path: &Path) -> bool {
    super::subagent_git_command::success(path, &["rev-parse", "--is-inside-work-tree"])
        .await
        .unwrap_or(false)
}

pub async fn create(
    project: &Path,
    child_id: &str,
    execution_id: &str,
) -> Result<PathBuf, String> {
    super::subagent_directory_limits::validate_source(project).await?;
    let repository = repository_path(child_id, execution_id)?;
    let worktree = super::subagent_worktree::path_for_execution(child_id, execution_id)?;
    ensure_absent(&repository).await?;
    ensure_absent(&worktree).await?;
    let parent = repository
        .parent()
        .ok_or_else(generic_error)?;
    tokio::fs::create_dir_all(parent)
        .await
        .map_err(|_| generic_error())?;

    let initialized = Command::new("git")
        .args(["init", "--bare"])
        .arg(&repository)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .status()
        .await
        .map(|status| status.success())
        .map_err(|_| generic_error())?;
    if !initialized {
        cleanup_failed(child_id, execution_id).await;
        return Err(generic_error());
    }
    let added = run(&["add", "-A"], Some((project, &repository)), None).await?;
    let committed = if added {
        run(
            &[
                "-c",
                crate::services::brand::GIT_AUTHOR_NAME_CONFIG,
                "-c",
                crate::services::brand::GIT_AUTHOR_EMAIL_CONFIG,
                "commit",
                "--no-verify",
                "--allow-empty",
                "-m",
                crate::services::brand::DIRECTORY_BASELINE_COMMIT_MESSAGE,
            ],
            Some((project, &repository)),
            None,
        )
        .await?
    } else {
        false
    };
    if !committed || !super::subagent_directory_limits::repository_is_bounded(&repository).await {
        cleanup_failed(child_id, execution_id).await;
        return Err(generic_error());
    }
    let branch = super::subagent_worktree::branch_for_execution(execution_id)?;
    let checked_out = Command::new("git")
        .arg("--git-dir")
        .arg(&repository)
        .args(["worktree", "add", "-b", &branch])
        .arg(&worktree)
        .arg("HEAD")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .status()
        .await
        .map(|status| status.success())
        .map_err(|_| generic_error())?;
    if !checked_out {
        cleanup_failed(child_id, execution_id).await;
        return Err(generic_error());
    }
    if !worktree.is_dir() {
        cleanup_failed(child_id, execution_id).await;
        return Err(generic_error());
    }
    Ok(worktree)
}

pub fn repository_path(child_id: &str, execution_id: &str) -> Result<PathBuf, String> {
    super::types_subagent_change::validate_uuid(child_id)?;
    super::types_subagent_change::validate_uuid(execution_id)?;
    Ok(crate::services::paths::data_dir()
        .join("subagent-directory-repos")
        .join(child_id)
        .join(execution_id))
}

pub async fn remove_repository(child_id: &str, execution_id: &str) -> Result<(), String> {
    let path = repository_path(child_id, execution_id)?;
    match tokio::fs::symlink_metadata(&path).await {
        Ok(metadata) if metadata.file_type().is_symlink() => Err(generic_error()),
        Ok(_) => tokio::fs::remove_dir_all(path)
            .await
            .map_err(|_| generic_error()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err(generic_error()),
    }
}

async fn run(
    args: &[&str],
    work_tree: Option<(&Path, &Path)>,
    git_dir: Option<&Path>,
) -> Result<bool, String> {
    let mut command = Command::new("git");
    if let Some((work_tree, repository)) = work_tree {
        command.arg("--git-dir").arg(repository).arg("--work-tree").arg(work_tree);
    } else if let Some(repository) = git_dir {
        command.arg("--git-dir").arg(repository);
    }
    command.args(args).kill_on_drop(true);
    command.stdout(Stdio::null()).stderr(Stdio::null());
    command.status().await.map(|status| status.success()).map_err(|_| generic_error())
}

async fn ensure_absent(path: &Path) -> Result<(), String> {
    match tokio::fs::symlink_metadata(path).await {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        _ => Err(generic_error()),
    }
}

async fn cleanup_failed(child_id: &str, execution_id: &str) {
    let worktree = super::subagent_worktree::path_for_execution(child_id, execution_id).ok();
    if let Some(worktree) = worktree {
        let _ = super::subagent_worktree::remove_owned(
            &worktree.to_string_lossy(),
            child_id,
            execution_id,
        )
        .await;
    }
    let _ = remove_repository(child_id, execution_id).await;
}

fn generic_error() -> String {
    "Préparation du dossier isolé impossible".to_string()
}
