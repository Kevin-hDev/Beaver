use std::path::Path;

pub async fn capture(
    project_path: &Path,
    child_id: &str,
    execution_id: &str,
    worktree: &Path,
) -> Result<Option<String>, String> {
    let Some(meta) = super::subagent_git_run::capture(
        project_path,
        child_id,
        execution_id,
        worktree,
    )
    .await?
    else {
        return Ok(None);
    };
    let metadata = serde_json::to_string(&meta)
        .map_err(|_| "Métadonnées de changement indisponibles".to_string())?;
    Ok(Some(format!(
        "\n\n<subagent_change_metadata>\n{metadata}\n</subagent_change_metadata>"
    )))
}

pub async fn delete_empty_workspace(project_path: &Path, child_id: &str, execution_id: &str) {
    if super::subagent_directory_workspace::is_git_repository(project_path).await {
        let Ok(branch) = super::subagent_worktree::branch_for_execution(execution_id) else {
            return;
        };
        let _ = super::subagent_git_command::delete_branch(project_path, &branch).await;
    } else {
        let _ = super::subagent_directory_workspace::remove_repository(child_id, execution_id).await;
    }
}

pub async fn cleanup_execution(
    project_path: &Path,
    child_id: &str,
    execution_id: &str,
    worktree_path: Option<&str>,
    retain_change: bool,
) {
    super::subagent_working_dir::cleanup_owned(child_id, execution_id, worktree_path).await;
    if !retain_change {
        delete_empty_workspace(project_path, child_id, execution_id).await;
    }
}

pub async fn recover_and_remove_orphan(
    session: &super::types_session::AgentSession,
) -> Result<(), String> {
    let Some(worktree) = session.subagent_worktree.as_deref() else {
        return Ok(());
    };
    if session.subagent_type.as_deref() != Some("coder") {
        return super::subagent_worktree::remove_for_child(worktree, &session.id).await;
    }
    let identity = super::subagent_worktree_identity::ManagedWorktreeIdentity::parse(worktree)?;
    identity.require_child(&session.id)?;
    let saved_project = super::project_store::list()
        .await
        .unwrap_or_default()
        .into_iter()
        .find(|project| Some(project.id.as_str()) == session.project_id.as_deref());
    let project_path = saved_project
        .map(|project| std::path::PathBuf::from(project.path))
        .filter(|path| path.is_dir())
        .or_else(|| {
            let path = std::path::PathBuf::from(&session.working_dir);
            path.is_dir().then_some(path)
        });
    let mut retain_branch = false;
    if let Some(project) = project_path.as_deref() {
        retain_branch = capture(
            project,
            &session.id,
            &identity.execution_id,
            &identity.path,
        )
        .await
        .ok()
        .flatten()
        .is_some();
    }
    super::subagent_worktree::remove_for_child(worktree, &session.id).await?;
    if !retain_branch {
        if let Some(project) = project_path.as_deref() {
            delete_empty_workspace(project, &session.id, &identity.execution_id).await;
        } else {
            let _ = super::subagent_directory_workspace::remove_repository(
                &session.id,
                &identity.execution_id,
            )
            .await;
        }
    }
    Ok(())
}
