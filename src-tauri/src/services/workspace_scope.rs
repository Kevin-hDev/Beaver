use crate::services::agent_local::types_session::AgentSessionMeta;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

const MAX_ANCESTORS: usize = 128;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", content = "id", rename_all = "snake_case")]
pub enum WorkspaceScope {
    Project(String),
    Session(String),
    #[default]
    Legacy,
}

pub async fn resolve(session_id: &str) -> Result<WorkspaceScope, String> {
    crate::services::agent_local::session_store::validate_session_id(session_id)?;
    let metas = crate::services::agent_local::session_index::read_index().await?;
    let scope = resolve_from_metas(session_id, &metas)?;
    if let WorkspaceScope::Project(project_id) = &scope {
        if crate::services::agent_local::project_store::find(project_id)
            .await?
            .is_none()
        {
            return Err(scope_error());
        }
    }
    Ok(scope)
}

pub(crate) fn resolve_from_metas(
    session_id: &str,
    metas: &[AgentSessionMeta],
) -> Result<WorkspaceScope, String> {
    let mut current_id = session_id;
    let mut seen = HashSet::with_capacity(MAX_ANCESTORS);
    for _ in 0..MAX_ANCESTORS {
        if !seen.insert(current_id) {
            return Err(scope_error());
        }
        let current = metas
            .iter()
            .find(|meta| meta.id == current_id)
            .ok_or_else(scope_error)?;
        if let Some(project_id) = current.project_id.as_ref() {
            return Ok(WorkspaceScope::Project(project_id.clone()));
        }
        let parent = current
            .clone_root_session_id
            .as_deref()
            .or(current.clone_parent_session_id.as_deref())
            .or(current.parent_session_id.as_deref());
        match parent {
            Some(parent_id) => current_id = parent_id,
            None => return Ok(WorkspaceScope::Session(current.id.clone())),
        }
    }
    Err(scope_error())
}

fn scope_error() -> String {
    "Espace de travail indisponible".into()
}
