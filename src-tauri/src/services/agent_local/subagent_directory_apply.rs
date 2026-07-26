use super::types_subagent_change::{SubagentChangeMeta, SubagentChangeStatus};
use chrono::Utc;
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;

const MAX_CURRENT_FILE_BYTES: u64 = 64 * 1024 * 1024;

pub async fn apply(
    project: &Path,
    mut meta: SubagentChangeMeta,
) -> Result<SubagentChangeMeta, String> {
    if meta.status == SubagentChangeStatus::Applied {
        cleanup_repository(&meta).await;
        return Ok(meta);
    }
    if meta.paths_truncated || !matches!(meta.status, SubagentChangeStatus::Pending | SubagentChangeStatus::Conflict) {
        return Err(generic_error());
    }
    let repository = super::subagent_directory_change::repository(&meta)?;
    if !baseline_is_current(project, &repository, &meta).await? {
        meta.status = SubagentChangeStatus::Conflict;
        meta.updated_at = Utc::now();
        super::subagent_change_store::save(&meta).await?;
        return Err("Le changement entre en conflit".into());
    }
    let stage_id = uuid::Uuid::new_v4().to_string();
    let stage = super::subagent_worktree::path_for_execution(&meta.child_session_id, &stage_id)?;
    checkout(&repository, &meta.commit, &stage).await?;
    let applied = super::subagent_directory_transaction::apply(project, &stage, &meta.changed_paths).await;
    let cleanup = super::subagent_worktree::remove_owned(
        &stage.to_string_lossy(),
        &meta.child_session_id,
        &stage_id,
    )
    .await;
    if applied.is_err() || cleanup.is_err() {
        return Err(generic_error());
    }
    meta.status = SubagentChangeStatus::Applied;
    meta.updated_at = Utc::now();
    meta.applied_commit = Some(meta.commit.clone());
    super::subagent_change_store::save(&meta).await?;
    cleanup_repository(&meta).await;
    Ok(meta)
}

pub async fn discard(mut meta: SubagentChangeMeta) -> Result<SubagentChangeMeta, String> {
    if meta.status == SubagentChangeStatus::Discarded {
        cleanup_repository(&meta).await;
        return Ok(meta);
    }
    if !matches!(meta.status, SubagentChangeStatus::Pending | SubagentChangeStatus::Conflict) {
        return Err(generic_error());
    }
    meta.status = SubagentChangeStatus::Discarded;
    meta.updated_at = Utc::now();
    super::subagent_change_store::save(&meta).await?;
    cleanup_repository(&meta).await;
    Ok(meta)
}

async fn baseline_is_current(
    project: &Path,
    repository: &Path,
    meta: &SubagentChangeMeta,
) -> Result<bool, String> {
    for changed in &meta.changed_paths {
        let target = project.join(&changed.path);
        let baseline = baseline_oid(repository, &meta.base_commit, &changed.path).await?;
        let current = current_oid(&target).await?;
        if baseline != current {
            return Ok(false);
        }
    }
    Ok(true)
}

async fn baseline_oid(
    repository: &Path,
    commit: &str,
    path: &str,
) -> Result<Option<String>, String> {
    let output = Command::new("git")
        .arg("--git-dir")
        .arg(repository)
        .args(["ls-tree", "-z", commit, "--", path])
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .output()
        .await
        .map_err(|_| generic_error())?;
    if !output.status.success() || output.stdout.len() > 1_024 {
        return Err(generic_error());
    }
    if output.stdout.is_empty() {
        return Ok(None);
    }
    let text = String::from_utf8(output.stdout).map_err(|_| generic_error())?;
    let header = text.split('\t').next().ok_or_else(generic_error)?;
    let mut fields = header.split_whitespace();
    let mode = fields.next().ok_or_else(generic_error)?;
    let kind = fields.next().ok_or_else(generic_error)?;
    let oid = fields.next().ok_or_else(generic_error)?;
    if kind != "blob" || mode == "120000" || oid.len() != 40 {
        return Err(generic_error());
    }
    Ok(Some(oid.to_string()))
}

async fn current_oid(path: &Path) -> Result<Option<String>, String> {
    let metadata = match tokio::fs::symlink_metadata(path).await {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err(generic_error()),
    };
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > MAX_CURRENT_FILE_BYTES {
        return Err(generic_error());
    }
    let output = Command::new("git")
        .arg("hash-object")
        .arg(path)
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .output()
        .await
        .map_err(|_| generic_error())?;
    if !output.status.success() || output.stdout.len() > 64 {
        return Err(generic_error());
    }
    String::from_utf8(output.stdout)
        .map(|value| Some(value.trim().to_string()))
        .map_err(|_| generic_error())
}

async fn checkout(repository: &Path, commit: &str, stage: &Path) -> Result<(), String> {
    let status = Command::new("git")
        .arg("--git-dir")
        .arg(repository)
        .args(["worktree", "add", "--detach"])
        .arg(stage)
        .arg(commit)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .status()
        .await
        .map_err(|_| generic_error())?;
    status.success().then_some(()).ok_or_else(generic_error)
}

async fn cleanup_repository(meta: &SubagentChangeMeta) {
    let Ok(execution) = super::subagent_directory_change::execution_id(meta) else { return };
    if super::subagent_directory_workspace::remove_repository(&meta.child_session_id, execution)
        .await
        .is_err()
    {
        eprintln!("[subagent] directory repository cleanup failed");
    }
}

fn generic_error() -> String {
    "Application du changement impossible".to_string()
}
