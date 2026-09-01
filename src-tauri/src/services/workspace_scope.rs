use crate::services::agent_local::types_session::AgentSessionMeta;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

const MAX_ANCESTORS: usize = 128;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(tag = "kind", content = "id", rename_all = "snake_case")]
#[cfg_attr(test, ts(tag = "kind", content = "id", rename_all = "snake_case"))]
pub enum WorkspaceScope {
    Project(String),
    Session(String),
    #[default]
    Legacy,
}

pub(crate) struct WorkspaceSnapshot {
    metas: Vec<AgentSessionMeta>,
    project_ids: HashSet<String>,
}

impl WorkspaceSnapshot {
    pub(crate) async fn load() -> Result<Self, String> {
        let metas = crate::services::agent_local::session_index::read_index().await?;
        let project_ids = crate::services::agent_local::project_store::list()
            .await?
            .into_iter()
            .map(|project| project.id)
            .collect();
        Ok(Self { metas, project_ids })
    }

    pub(crate) fn resolve(&self, session_id: &str) -> Result<WorkspaceScope, String> {
        resolve_from_snapshot(session_id, &self.metas, &self.project_ids)
    }

    pub(crate) fn contains_session(&self, session_id: &str) -> bool {
        self.metas.iter().any(|meta| meta.id == session_id)
    }

    pub(crate) fn is_live(&self, workspace: &WorkspaceScope) -> bool {
        match workspace {
            WorkspaceScope::Project(id) => self.project_ids.contains(id),
            WorkspaceScope::Session(id) => self.resolve(id).as_ref() == Ok(workspace),
            WorkspaceScope::Legacy => true,
        }
    }
}

pub async fn resolve(session_id: &str) -> Result<WorkspaceScope, String> {
    crate::services::agent_local::session_store::validate_session_id(session_id)?;
    WorkspaceSnapshot::load().await?.resolve(session_id)
}

#[cfg(test)]
pub(crate) fn resolve_from_metas(
    session_id: &str,
    metas: &[AgentSessionMeta],
) -> Result<WorkspaceScope, String> {
    let project_ids = metas
        .iter()
        .filter_map(|meta| meta.project_id.clone())
        .collect();
    resolve_from_snapshot(session_id, metas, &project_ids)
}

fn resolve_from_snapshot(
    session_id: &str,
    metas: &[AgentSessionMeta],
    project_ids: &HashSet<String>,
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
        if let Some(project_id) = current
            .project_id
            .as_ref()
            .filter(|project_id| project_ids.contains(*project_id))
        {
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
