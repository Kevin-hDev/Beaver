use super::types_subagent_change::{
    SubagentChangeMeta, SubagentChangeStatus, SubagentWorkspaceKind,
};
use chrono::Utc;
use std::path::Path;
use std::process::Stdio;

const DIRECTORY_TARGET: &str = "cl-go/directory";
pub const DIRECTORY_PROJECT: &str = "session-directory";

pub async fn capture(
    project_path: &Path,
    child_id: &str,
    execution_id: &str,
    worktree: &Path,
) -> Result<Option<SubagentChangeMeta>, String> {
    let _guard = super::subagent_git_lock::acquire(project_path).await?;
    let child = validate_child(child_id).await?;
    let project_id = child.project_id.unwrap_or_else(|| DIRECTORY_PROJECT.into());
    let branch = super::subagent_worktree::branch_for_execution(execution_id)?;
    if super::subagent_directory_git::text(worktree, &["branch", "--show-current"]).await?
        != branch
    {
        return Err(generic_error());
    }
    let existing = super::subagent_change_store::load_optional(child_id).await?;
    if existing.as_ref().is_some_and(|meta| {
        meta.workspace_kind != SubagentWorkspaceKind::Directory || meta.branch != branch
    }) {
        return Err(generic_error());
    }
    let current_head =
        super::subagent_directory_git::text(worktree, &["rev-parse", "HEAD"]).await?;
    let base_commit = existing
        .as_ref()
        .map(|meta| meta.base_commit.clone())
        .unwrap_or(current_head);
    if !super::subagent_directory_git::success(worktree, &["add", "-A"]).await? {
        return Err(generic_error());
    }
    if super::subagent_directory_git::success(worktree, &["diff", "--cached", "--quiet"])
        .await?
    {
        return Ok(existing);
    }
    if existing.is_some()
        && !super::subagent_directory_git::success(
            worktree,
            &["reset", "--soft", &base_commit],
        )
        .await?
    {
        return Err(generic_error());
    }
    let changed = changed_paths(worktree).await?;
    let id = existing
        .as_ref()
        .map(|meta| meta.id.clone())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    commit_snapshot(worktree, &id).await?;
    let commit = super::subagent_directory_git::text(worktree, &["rev-parse", "HEAD"]).await?;
    let now = Utc::now();
    let meta = SubagentChangeMeta {
        id,
        child_session_id: child_id.to_string(),
        project_id,
        base_commit,
        commit,
        branch,
        target_branch: DIRECTORY_TARGET.into(),
        workspace_kind: SubagentWorkspaceKind::Directory,
        changed_paths: changed.0,
        paths_truncated: changed.1,
        status: SubagentChangeStatus::Pending,
        created_at: existing.map(|meta| meta.created_at).unwrap_or(now),
        updated_at: now,
        applied_commit: None,
    };
    super::subagent_change_store::save(&meta).await?;
    Ok(Some(meta))
}

pub async fn patch(meta: &SubagentChangeMeta) -> Result<String, String> {
    let repository = repository(meta)?;
    let output = super::subagent_directory_git::command()
        .arg("--git-dir")
        .arg(repository)
        .args(["show", "--format=", "--binary", &meta.commit])
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .output()
        .await
        .map_err(|_| generic_error())?;
    if !output.status.success() || output.stdout.len() > 256 * 1024 {
        return Err(generic_error());
    }
    String::from_utf8(output.stdout).map_err(|_| generic_error())
}

pub fn repository(meta: &SubagentChangeMeta) -> Result<std::path::PathBuf, String> {
    let execution = execution_id(meta)?;
    super::subagent_directory_workspace::repository_path(&meta.child_session_id, execution)
}

pub fn execution_id(meta: &SubagentChangeMeta) -> Result<&str, String> {
    let execution = meta
        .branch
        .strip_prefix("cl-go/subagent/")
        .ok_or_else(generic_error)?;
    super::types_subagent_change::validate_uuid(execution)?;
    Ok(execution)
}

async fn validate_child(child_id: &str) -> Result<super::types_session::AgentSession, String> {
    super::types_subagent_change::validate_uuid(child_id)?;
    let child = super::session_store::get(child_id).await.map_err(|_| generic_error())?;
    if child.subagent_type.as_deref() != Some("coder") {
        return Err(generic_error());
    }
    Ok(child)
}

async fn commit_snapshot(worktree: &Path, id: &str) -> Result<(), String> {
    let message = crate::services::brand::directory_change_commit_message(id);
    let status = super::subagent_directory_git::command()
        .args(["-C"])
        .arg(worktree)
        .args([
            "-c",
            crate::services::brand::GIT_AUTHOR_NAME_CONFIG,
            "-c",
            crate::services::brand::GIT_AUTHOR_EMAIL_CONFIG,
            "commit", "--no-verify", "-m",
        ])
        .arg(message)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .status()
        .await
        .map_err(|_| generic_error())?;
    status.success().then_some(()).ok_or_else(generic_error)
}

fn generic_error() -> String {
    "Capture du changement impossible".to_string()
}

async fn changed_paths(
    worktree: &Path,
) -> Result<(Vec<super::types_subagent_change::SubagentChangedPath>, bool), String> {
    let output = super::subagent_directory_git::output(
        worktree,
        &["diff", "--cached", "--name-status", "--no-renames", "-z"],
    )
    .await?;
    super::subagent_git_run::parse_changed_paths(&output)
}
