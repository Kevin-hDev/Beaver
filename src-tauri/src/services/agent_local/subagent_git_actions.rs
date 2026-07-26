use super::types_subagent_change::{
    SubagentChangeMeta, SubagentChangeStatus, SubagentWorkspaceKind,
};
use chrono::Utc;
use std::path::Path;

const MAX_PATCH_CHARS: usize = 12_000;
const MAX_STATUS_BYTES: usize = 1024 * 1024;

pub async fn inspect(
    project_path: &Path,
    parent_id: &str,
    child_id: &str,
    change_id: &str,
) -> Result<(SubagentChangeMeta, String, bool), String> {
    let meta = owned_change(parent_id, child_id, change_id).await?;
    validate_project(project_path, parent_id, &meta).await?;
    let patch = match meta.workspace_kind {
        SubagentWorkspaceKind::Git => {
            super::subagent_git_command::text(
                project_path,
                &["show", "--format=", "--binary", &meta.commit],
            )
            .await?
        }
        SubagentWorkspaceKind::Directory => {
            super::subagent_directory_change::patch(&meta).await?
        }
    };
    let truncated = patch.chars().count() > MAX_PATCH_CHARS;
    let patch = patch.chars().take(MAX_PATCH_CHARS).collect();
    Ok((meta, patch, truncated))
}

pub async fn apply(
    project_path: &Path,
    parent_id: &str,
    child_id: &str,
    change_id: &str,
) -> Result<SubagentChangeMeta, String> {
    let _guard = super::subagent_git_lock::acquire(project_path).await?;
    let mut meta = owned_change(parent_id, child_id, change_id).await?;
    validate_project(project_path, parent_id, &meta).await?;
    if meta.workspace_kind == SubagentWorkspaceKind::Directory {
        return super::subagent_directory_apply::apply(project_path, meta).await;
    }
    if meta.status == SubagentChangeStatus::Applied {
        super::subagent_git_command::delete_branch(project_path, &meta.branch).await?;
        return Ok(meta);
    }
    if !matches!(meta.status, SubagentChangeStatus::Pending | SubagentChangeStatus::Conflict) {
        return Err("Dépôt parent non prêt".into());
    }
    let status_before = status_snapshot(project_path, &[]).await?;
    if !super::subagent_git_command::success(project_path, &["diff", "--cached", "--quiet"]).await?
        || !changed_paths_are_clean(project_path, &meta).await?
    {
        return Err("Dépôt parent non prêt".into());
    }
    let target = super::subagent_git_command::text(project_path, &["branch", "--show-current"]).await?;
    if target != meta.target_branch {
        return Err("Branche cible incompatible".into());
    }
    let head = super::subagent_git_command::text(project_path, &["rev-parse", "HEAD"]).await?;
    if !super::subagent_git_command::cherry_pick(project_path, &meta.commit).await? {
        let aborted = super::subagent_git_command::success(project_path, &["cherry-pick", "--abort"]).await?;
        let restored = super::subagent_git_command::text(project_path, &["rev-parse", "HEAD"]).await? == head
            && status_snapshot(project_path, &[]).await? == status_before;
        if !aborted || !restored {
            return Err("Restauration du dépôt parent impossible".into());
        }
        meta.status = SubagentChangeStatus::Conflict;
        meta.updated_at = Utc::now();
        super::subagent_change_store::save(&meta).await?;
        return Err("Le changement entre en conflit".into());
    }
    meta.status = SubagentChangeStatus::Applied;
    meta.updated_at = Utc::now();
    meta.applied_commit = Some(
        super::subagent_git_command::text(project_path, &["rev-parse", "HEAD"]).await?,
    );
    super::subagent_change_store::save(&meta).await?;
    super::subagent_git_command::delete_branch(project_path, &meta.branch).await?;
    Ok(meta)
}

pub async fn discard(
    project_path: &Path,
    parent_id: &str,
    child_id: &str,
    change_id: &str,
) -> Result<SubagentChangeMeta, String> {
    let _guard = super::subagent_git_lock::acquire(project_path).await?;
    let mut meta = owned_change(parent_id, child_id, change_id).await?;
    validate_project(project_path, parent_id, &meta).await?;
    if meta.workspace_kind == SubagentWorkspaceKind::Directory {
        return super::subagent_directory_apply::discard(meta).await;
    }
    if meta.status == SubagentChangeStatus::Discarded {
        super::subagent_git_command::delete_branch(project_path, &meta.branch).await?;
        return Ok(meta);
    }
    if !matches!(meta.status, SubagentChangeStatus::Pending | SubagentChangeStatus::Conflict) {
        return Err("Changement non abandonnable".into());
    }
    super::subagent_git_command::delete_branch(project_path, &meta.branch).await?;
    meta.status = SubagentChangeStatus::Discarded;
    meta.updated_at = Utc::now();
    super::subagent_change_store::save(&meta).await?;
    Ok(meta)
}

async fn owned_change(
    parent_id: &str,
    child_id: &str,
    change_id: &str,
) -> Result<SubagentChangeMeta, String> {
    super::types_subagent_change::validate_uuid(parent_id)?;
    super::types_subagent_change::validate_uuid(child_id)?;
    super::types_subagent_change::validate_uuid(change_id)?;
    let child = super::session_store::get(child_id)
        .await
        .map_err(|_| "Changement sous-agent indisponible".to_string())?;
    if child.parent_session_id.as_deref() != Some(parent_id)
        || child.subagent_type.as_deref() != Some("coder")
    {
        return Err("Changement sous-agent indisponible".into());
    }
    let meta = super::subagent_change_store::load(child_id).await?;
    if meta.id != change_id {
        return Err("Changement sous-agent indisponible".into());
    }
    Ok(meta)
}

async fn validate_project(
    project_path: &Path,
    parent_id: &str,
    meta: &SubagentChangeMeta,
) -> Result<(), String> {
    let child = super::session_store::get(&meta.child_session_id)
        .await
        .map_err(|_| "Projet sous-agent indisponible".to_string())?;
    let parent = super::session_store::get(parent_id)
        .await
        .map_err(|_| "Projet sous-agent indisponible".to_string())?;
    if !project_path.is_dir() {
        return Err("Projet sous-agent indisponible".into());
    }
    if meta.workspace_kind == SubagentWorkspaceKind::Git {
        if child.project_id.as_deref() != Some(&meta.project_id) {
            return Err("Projet sous-agent indisponible".into());
        }
        return Ok(());
    }
    let expected = std::path::Path::new(&parent.working_dir)
        .canonicalize()
        .map_err(|_| "Projet sous-agent indisponible".to_string())?;
    let actual = project_path
        .canonicalize()
        .map_err(|_| "Projet sous-agent indisponible".to_string())?;
    let identity_matches = parent.project_id.as_deref() == Some(&meta.project_id)
        || parent.project_id.is_none()
            && meta.project_id == super::subagent_directory_change::DIRECTORY_PROJECT;
    if !identity_matches || actual != expected {
        return Err("Projet sous-agent indisponible".into());
    }
    Ok(())
}

async fn changed_paths_are_clean(
    project_path: &Path,
    meta: &SubagentChangeMeta,
) -> Result<bool, String> {
    let paths = meta
        .changed_paths
        .iter()
        .map(|changed| changed.path.as_str())
        .collect::<Vec<_>>();
    Ok(status_snapshot(project_path, &paths).await?.is_empty())
}

async fn status_snapshot(project_path: &Path, paths: &[&str]) -> Result<Vec<u8>, String> {
    let mut args = vec!["status", "--porcelain=v1", "-z", "--untracked-files=all"];
    if !paths.is_empty() {
        args.push("--");
        args.extend_from_slice(paths);
    }
    let output = super::subagent_git_command::output(project_path, &args).await?;
    if !output.status.success() || output.stdout.len() > MAX_STATUS_BYTES {
        return Err("Dépôt parent non prêt".into());
    }
    Ok(output.stdout)
}
