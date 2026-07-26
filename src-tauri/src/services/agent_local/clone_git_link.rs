use super::{clone_git_checks::ensure_clone_belongs_to_root_action, session_store, session_tabs};
use crate::services::git::{action_error::GitActionError, branch, branch_delete};
use std::path::Path;

pub async fn link_existing_branch(
    root_session_id: &str,
    clone_session_id: &str,
    repo_path: &Path,
    branch_name: &str,
) -> Result<session_tabs::SessionTabs, GitActionError> {
    branch::validate_branch_name(branch_name).map_err(|_| GitActionError::BranchUnavailable)?;
    if !branch_delete::branch_exists(repo_path, branch_name)? {
        return Err(GitActionError::BranchUnavailable);
    }

    let mut clone = session_store::get(clone_session_id)
        .await
        .map_err(|_| GitActionError::CloneUnavailable)?;
    ensure_clone_belongs_to_root_action(&clone, root_session_id).await?;
    clone.git_branch = Some(branch_name.to_string());
    session_store::save(&clone)
        .await
        .map_err(|_| GitActionError::InternalError)?;
    session_tabs::set_clone_git_branch(
        root_session_id,
        clone_session_id,
        Some(branch_name.to_string()),
    )
    .await
    .map_err(|_| GitActionError::InternalError)
}

#[cfg(test)]
#[path = "clone_git_link_tests.rs"]
mod tests;
