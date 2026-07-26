use super::types_subagent_change::{SubagentChangeStatus, SubagentWorkspaceKind};
use chrono::Utc;
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;

pub async fn seed_pending(
    child_id: &str,
    execution_id: &str,
    worktree: &Path,
) -> Result<(), String> {
    let Some(mut meta) = super::subagent_change_store::load_optional(child_id).await? else {
        return Ok(());
    };
    if !matches!(meta.status, SubagentChangeStatus::Pending | SubagentChangeStatus::Conflict) {
        return Ok(());
    }
    if meta.workspace_kind != SubagentWorkspaceKind::Directory {
        return Err(generic_error());
    }
    let old_execution = super::subagent_directory_change::execution_id(&meta)?.to_string();
    if old_execution == execution_id {
        return Ok(());
    }
    let old_repository = super::subagent_directory_change::repository(&meta)?;
    let base_commit = super::subagent_git_command::text(worktree, &["rev-parse", "HEAD"]).await?;
    let fetched = Command::new("git")
        .args(["-C"])
        .arg(worktree)
        .args(["fetch"])
        .arg(&old_repository)
        .arg(&meta.branch)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .status()
        .await
        .map_err(|_| generic_error())?;
    if !fetched.success()
        || !super::subagent_git_command::cherry_pick(worktree, &meta.commit).await?
    {
        let _ = super::subagent_git_command::success(worktree, &["cherry-pick", "--abort"]).await;
        return Err("Le changement précédent entre en conflit".into());
    }
    meta.branch = super::subagent_worktree::branch_for_execution(execution_id)?;
    meta.base_commit = base_commit;
    meta.commit = super::subagent_git_command::text(worktree, &["rev-parse", "HEAD"]).await?;
    meta.status = SubagentChangeStatus::Pending;
    meta.updated_at = Utc::now();
    super::subagent_change_store::save(&meta).await?;
    if super::subagent_directory_workspace::remove_repository(child_id, &old_execution)
        .await
        .is_err()
    {
        eprintln!("[subagent] previous directory repository cleanup failed");
    }
    Ok(())
}

fn generic_error() -> String {
    "Préparation du dossier isolé impossible".to_string()
}
