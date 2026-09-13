use std::path::Path;

#[cfg(test)]
static FAIL_CAPTURE_CHILDREN: tokio::sync::Mutex<Vec<String>> =
    tokio::sync::Mutex::const_new(Vec::new());

#[derive(serde::Serialize)]
struct PromptChangeMetadata<'a> {
    subagent_id: &'a str,
    change_id: &'a str,
    #[serde(flatten)]
    change: &'a super::types_subagent_change::SubagentChangeMeta,
}

pub async fn capture(
    project_path: &Path,
    child_id: &str,
    execution_id: &str,
    worktree: &Path,
) -> Result<Option<String>, String> {
    #[cfg(test)]
    {
        let mut children = FAIL_CAPTURE_CHILDREN.lock().await;
        if let Some(index) = children.iter().position(|candidate| candidate == child_id) {
            children.swap_remove(index);
            return Err("injected capture failure".to_string());
        }
    }
    let Some(meta) =
        super::subagent_git_run::capture(project_path, child_id, execution_id, worktree).await?
    else {
        return Ok(None);
    };
    let metadata = serialize_prompt_metadata(&meta)?;
    Ok(Some(format!(
        "\n\n<subagent_change_metadata>\n{metadata}\n</subagent_change_metadata>"
    )))
}

fn serialize_prompt_metadata(
    change: &super::types_subagent_change::SubagentChangeMeta,
) -> Result<String, String> {
    serde_json::to_string(&PromptChangeMetadata {
        subagent_id: &change.child_session_id,
        change_id: &change.id,
        change,
    })
    .map_err(|_| "Métadonnées de changement indisponibles".to_string())
}

pub async fn delete_empty_workspace(project_path: &Path, child_id: &str, execution_id: &str) {
    if super::subagent_directory_workspace::is_git_repository(project_path).await {
        let Ok(branch) = super::subagent_worktree::branch_for_execution(execution_id) else {
            return;
        };
        let _ = super::subagent_git_command::delete_branch(project_path, &branch).await;
    } else {
        let _ =
            super::subagent_directory_workspace::remove_repository(child_id, execution_id).await;
    }
}

pub async fn cleanup_execution(
    project_path: &Path,
    child_id: &str,
    execution_id: &str,
    worktree_path: Option<&str>,
    retain_change: bool,
    retain_worktree: bool,
) {
    if retain_worktree {
        return;
    }
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
    identity.reject_symlinks().await?;
    match tokio::fs::symlink_metadata(&identity.path).await {
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(_) => return Err("Chemin worktree invalide".to_string()),
    }
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
    let project = project_path
        .as_deref()
        .ok_or_else(|| "Projet sous-agent indisponible".to_string())?;
    let retain_branch = capture(project, &session.id, &identity.execution_id, &identity.path)
        .await?
        .is_some();
    super::subagent_worktree::remove_for_child(worktree, &session.id).await?;
    if !retain_branch {
        delete_empty_workspace(project, &session.id, &identity.execution_id).await;
    }
    Ok(())
}

#[cfg(test)]
pub(super) async fn fail_next_capture_for_child(child_id: &str) {
    let mut children = FAIL_CAPTURE_CHILDREN.lock().await;
    if children.iter().any(|candidate| candidate == child_id) {
        return;
    }
    assert!(children.len() < 16, "capture failure seam capacity");
    children.push(child_id.to_string());
}

#[cfg(test)]
#[path = "subagent_task_change_tests.rs"]
mod tests;
