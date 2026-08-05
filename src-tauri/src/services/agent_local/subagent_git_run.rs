use super::types_subagent_change::{
    SubagentChangeMeta, SubagentChangeStatus, SubagentChangedPath, MAX_CHANGED_PATHS,
};
use chrono::Utc;
use std::path::Path;

#[cfg(test)]
pub async fn seed_pending(
    project_path: &Path,
    child_id: &str,
    execution_id: &str,
    worktree: &Path,
) -> Result<(), String> {
    let _guard = super::subagent_git_lock::acquire(project_path).await?;
    seed_pending_locked(project_path, child_id, execution_id, worktree).await
}

pub async fn seed_pending_locked(
    project_path: &Path,
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
    let target_branch = super::subagent_git_command::text(project_path, &["branch", "--show-current"]).await?;
    if target_branch != meta.target_branch {
        return Err("Branche cible incompatible".into());
    }
    let base = super::subagent_git_command::text(project_path, &["rev-parse", "HEAD"]).await?;
    if !super::subagent_git_command::cherry_pick(worktree, &meta.commit).await? {
        let _ = super::subagent_git_command::success(worktree, &["cherry-pick", "--abort"]).await;
        return Err("Le changement précédent entre en conflit".into());
    }
    let old_branch = meta.branch.clone();
    meta.branch = super::subagent_worktree::branch_for_execution(execution_id)?;
    meta.base_commit = base;
    meta.commit = super::subagent_git_command::text(worktree, &["rev-parse", "HEAD"]).await?;
    meta.status = SubagentChangeStatus::Pending;
    meta.updated_at = Utc::now();
    super::subagent_change_store::save(&meta).await?;
    if old_branch != meta.branch {
        super::subagent_git_command::delete_branch(project_path, &old_branch).await?;
    }
    Ok(())
}

pub async fn capture(
    project_path: &Path,
    child_id: &str,
    execution_id: &str,
    worktree: &Path,
) -> Result<Option<SubagentChangeMeta>, String> {
    if !super::subagent_directory_workspace::is_git_repository(project_path).await {
        return super::subagent_directory_change::capture(
            project_path,
            child_id,
            execution_id,
            worktree,
        )
        .await;
    }
    let _guard = super::subagent_git_lock::acquire(project_path).await?;
    super::types_subagent_change::validate_uuid(child_id)?;
    super::types_subagent_change::validate_uuid(execution_id)?;
    let child = super::session_store::get(child_id)
        .await
        .map_err(|_| "Session sous-agent indisponible".to_string())?;
    if child.subagent_type.as_deref() != Some("coder") {
        return Err("Profil sous-agent invalide".into());
    }
    let project_id = child
        .project_id
        .filter(|value| !value.is_empty() && value.chars().count() <= 128)
        .ok_or_else(|| "Projet sous-agent indisponible".to_string())?;
    let branch = super::subagent_worktree::branch_for_execution(execution_id)?;
    let actual_branch = super::subagent_git_command::text(worktree, &["branch", "--show-current"]).await?;
    if actual_branch != branch {
        return Err("Branche sous-agent invalide".into());
    }
    let existing = super::subagent_change_store::load_optional(child_id).await?;
    let current_head = super::subagent_git_command::text(worktree, &["rev-parse", "HEAD"]).await?;
    let base_commit = existing
        .as_ref()
        .filter(|meta| meta.branch == branch)
        .map(|meta| meta.base_commit.clone())
        .unwrap_or(current_head);
    let target_branch = super::subagent_git_command::text(project_path, &["branch", "--show-current"]).await?;
    if target_branch.is_empty() {
        return Err("Branche parent indisponible".into());
    }
    if !super::subagent_git_command::success(worktree, &["add", "-A"]).await? {
        return Err("Capture du changement impossible".into());
    }
    if super::subagent_git_command::success(worktree, &["diff", "--cached", "--quiet"]).await? {
        return Ok(existing.filter(|meta| meta.branch == branch));
    }
    if existing.as_ref().is_some_and(|meta| meta.branch == branch)
        && !super::subagent_git_command::success(worktree, &["reset", "--soft", &base_commit]).await?
    {
        return Err("Capture du changement impossible".into());
    }
    let changed = changed_paths(worktree).await?;
    let id = existing
        .as_ref()
        .filter(|meta| meta.branch == branch)
        .map(|meta| meta.id.clone())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let message = crate::services::brand::subagent_change_commit_message(&id);
    let committed = crate::services::background_command::new_tokio("git")
        .args(["-C"])
        .arg(worktree)
        .args([
            "-c",
            crate::services::brand::GIT_AUTHOR_NAME_CONFIG,
            "-c",
            crate::services::brand::GIT_AUTHOR_EMAIL_CONFIG,
            "commit",
            "--no-verify",
            "-m",
        ])
        .arg(message)
        .kill_on_drop(true)
        .output()
        .await
        .map_err(|_| "Capture du changement impossible".to_string())?;
    if !committed.status.success() {
        return Err("Capture du changement impossible".into());
    }
    let commit = super::subagent_git_command::text(worktree, &["rev-parse", "HEAD"]).await?;
    let now = Utc::now();
    let meta = SubagentChangeMeta {
        id,
        child_session_id: child_id.to_string(),
        project_id,
        base_commit,
        commit,
        branch,
        target_branch,
        workspace_kind: super::types_subagent_change::SubagentWorkspaceKind::Git,
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

pub(super) async fn changed_paths(worktree: &Path) -> Result<(Vec<SubagentChangedPath>, bool), String> {
    let output = super::subagent_git_command::output(
        worktree,
        &["diff", "--cached", "--name-status", "--no-renames", "-z"],
    )
    .await?;
    parse_changed_paths(&output)
}

pub(super) fn parse_changed_paths(
    output: &std::process::Output,
) -> Result<(Vec<SubagentChangedPath>, bool), String> {
    if !output.status.success() {
        return Err("Liste des changements indisponible".into());
    }
    let fields = output.stdout.split(|byte| *byte == 0).filter(|part| !part.is_empty()).collect::<Vec<_>>();
    let mut paths = Vec::new();
    let mut index = 0;
    while index + 1 < fields.len() && paths.len() < MAX_CHANGED_PATHS {
        let kind = String::from_utf8_lossy(fields[index]).to_string();
        index += 1;
        if kind.starts_with('R') || kind.starts_with('C') {
            index += 1;
        }
        let Some(raw_path) = fields.get(index) else { break };
        paths.push(SubagentChangedPath {
            path: String::from_utf8_lossy(raw_path).to_string(),
            kind: kind.chars().next().unwrap_or('M').to_string(),
        });
        index += 1;
    }
    let truncated = index < fields.len();
    Ok((paths, truncated))
}
