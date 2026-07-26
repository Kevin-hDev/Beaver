use super::{clone_git_checks::clone_linked_branch, session_store, session_tabs};
use crate::services::git::{action_error::GitActionError, branch, branch_delete};
use std::path::Path;

pub async fn close_tab_with_branch_cleanup(
    root_session_id: &str,
    tab_id: &str,
    repo_path: &Path,
    fallback_branch: Option<&str>,
) -> Result<session_tabs::SessionTabs, GitActionError> {
    let tab = session_tabs::get_tab(root_session_id, tab_id)
        .await
        .map_err(|_| GitActionError::CloneUnavailable)?;
    let clone = session_store::get(&tab.session_id)
        .await
        .map_err(|_| GitActionError::CloneUnavailable)?;
    let Some(git_branch) = clone_linked_branch(&clone, root_session_id)
        .await
        .map_err(|_| GitActionError::CloneUnavailable)?
    else {
        return session_tabs::close_tab(root_session_id, tab_id)
            .await
            .map_err(|_| GitActionError::InternalError);
    };
    let linked_session_ids = linked_sessions_for_branch(&git_branch, &tab.session_id).await?;

    let checkpoint = fallback_branch
        .map(str::to_string)
        .or(session_tabs::get_main_checkpoint_branch(root_session_id)
            .await
            .map_err(|_| GitActionError::InternalError)?);
    let repo_path = repo_path.to_path_buf();
    let deleted_branch = git_branch.clone();
    let replacement_branch = tokio::task::spawn_blocking(move || {
        cleanup_linked_branch(&repo_path, &deleted_branch, checkpoint.as_deref())
    })
    .await
    .map_err(|e| {
        eprintln!("[clone-git] close cleanup join: {e}");
        GitActionError::InternalError
    })??;

    unlink_branch_from_sessions(&linked_session_ids).await?;
    session_tabs::replace_main_checkpoint_branch(&git_branch, &replacement_branch)
        .await
        .map_err(|_| GitActionError::InternalError)?;
    session_tabs::close_tab(root_session_id, tab_id)
        .await
        .map_err(|_| GitActionError::InternalError)
}

async fn linked_sessions_for_branch(
    branch_name: &str,
    current_session_id: &str,
) -> Result<Vec<String>, GitActionError> {
    let mut ids: Vec<String> = super::session_index::read_index()
        .await
        .map_err(|_| GitActionError::InternalError)?
        .into_iter()
        .filter(|meta| meta.git_branch.as_deref() == Some(branch_name))
        .map(|meta| meta.id)
        .collect();
    if !ids.iter().any(|id| id == current_session_id) {
        ids.push(current_session_id.to_string());
    }
    Ok(ids)
}

async fn unlink_branch_from_sessions(session_ids: &[String]) -> Result<(), GitActionError> {
    for session_id in session_ids {
        let mut session = session_store::get(session_id)
            .await
            .map_err(|_| GitActionError::InternalError)?;
        session.git_branch = None;
        session_store::save(&session)
            .await
            .map_err(|_| GitActionError::InternalError)?;
    }
    session_tabs::clear_git_branch_for_sessions(session_ids)
        .await
        .map_err(|_| GitActionError::InternalError)
}

fn cleanup_linked_branch(
    repo_path: &Path,
    git_branch: &str,
    checkpoint: Option<&str>,
) -> Result<String, GitActionError> {
    if is_protected_base_branch(git_branch) {
        return Err(GitActionError::ProtectedBranch);
    }
    let context = branch::get_context(repo_path);
    if !context.is_git_repo {
        return Err(GitActionError::RepositoryUnavailable);
    }
    let replacement = if context.branch == git_branch {
        let fallback = choose_cleanup_fallback(repo_path, git_branch, checkpoint)?;
        branch::checkout_branch(repo_path, &fallback)?;
        fallback
    } else {
        context.branch
    };
    if branch_delete::branch_exists(repo_path, git_branch)? {
        branch_delete::delete_branch(repo_path, git_branch)?;
    }
    Ok(replacement)
}

fn choose_cleanup_fallback(
    repo_path: &Path,
    git_branch: &str,
    checkpoint: Option<&str>,
) -> Result<String, GitActionError> {
    for name in ["main", "master", "develop", "dev"] {
        if name != git_branch
            && branch_delete::branch_exists(repo_path, name)?
        {
            return Ok(name.to_string());
        }
    }
    if let Some(checkpoint) = checkpoint {
        if checkpoint != git_branch
            && branch_delete::branch_exists(repo_path, checkpoint)?
        {
            return Ok(checkpoint.to_string());
        }
    }
    let branch = branch::list_branches(repo_path)
        .map_err(|_| GitActionError::InternalError)?
        .into_iter()
        .find(|branch| branch.name != git_branch)
        .map(|branch| branch.name)
        .ok_or(GitActionError::NoFallbackBranch)?;
    Ok(branch)
}

fn is_protected_base_branch(branch_name: &str) -> bool {
    matches!(branch_name, "main" | "master" | "develop" | "dev")
}

#[cfg(test)]
#[path = "clone_git_cleanup_tests.rs"]
mod tests;
