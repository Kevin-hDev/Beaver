use crate::services::git::{action_error::GitActionError, branch};

pub(super) async fn ensure_clone_belongs_to_root(
    clone: &super::types_session::AgentSession,
    root_session_id: &str,
) -> Result<(), branch::CreateBranchError> {
    if clone_belongs_to_root(clone, root_session_id).await {
        Ok(())
    } else {
        Err(branch::CreateBranchError::InternalError)
    }
}

pub(super) async fn ensure_clone_belongs_to_root_string(
    clone: &super::types_session::AgentSession,
    root_session_id: &str,
) -> Result<(), String> {
    if clone_belongs_to_root(clone, root_session_id).await {
        Ok(())
    } else {
        Err("Action impossible".into())
    }
}

pub(super) async fn ensure_clone_belongs_to_root_action(
    clone: &super::types_session::AgentSession,
    root_session_id: &str,
) -> Result<(), GitActionError> {
    if clone_belongs_to_root(clone, root_session_id).await {
        Ok(())
    } else {
        Err(GitActionError::CloneUnavailable)
    }
}

pub(super) async fn clone_linked_branch(
    clone: &super::types_session::AgentSession,
    root_session_id: &str,
) -> Result<Option<String>, String> {
    ensure_clone_belongs_to_root_string(clone, root_session_id).await?;
    Ok(clone.git_branch.clone())
}

async fn clone_belongs_to_root(
    clone: &super::types_session::AgentSession,
    root_session_id: &str,
) -> bool {
    super::clone_roots::resolve_source_root_id(clone)
        .await
        .map(|root| root == root_session_id)
        .unwrap_or(false)
}

#[cfg(test)]
#[path = "clone_git_checks_tests.rs"]
mod tests;
