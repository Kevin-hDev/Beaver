use super::session_store::validate_session_id;
use super::types_session::AgentSessionMeta;
use std::collections::VecDeque;

const MAX_FAMILY_SCAN: usize = 4096;

pub async fn archive_family(id: &str) -> Result<(), String> {
    validate_session_id(id)?;
    let metas = super::session_index::read_index().await?;
    for session_id in archive_targets(&metas, id) {
        if session_id == id {
            super::session_archive::archive(&session_id).await?;
        } else {
            let _ = super::session_archive::archive(&session_id).await;
        }
    }
    Ok(())
}

pub async fn restore_with_parent(id: &str) -> Result<(), String> {
    validate_session_id(id)?;
    let metas = super::session_index::read_index().await?;
    let meta = metas
        .iter()
        .find(|entry| entry.id == id)
        .ok_or_else(generic_restore_error)?;
    let targets = restore_targets(&metas, meta)?;
    for ancestor in targets.iter().filter(|target| target.as_str() != id) {
        super::session_archive::restore(ancestor).await?;
    }
    super::session_archive::restore(id).await?;
    if meta.clone_parent_session_id.is_some() {
        let root_id = super::clone_roots::root_id_from_metas(
            &metas,
            &meta.id,
            meta.clone_parent_session_id.as_deref(),
        )?;
        super::session_tabs::ensure_clone_tab(&root_id, id).await?;
    }
    Ok(())
}

pub async fn delete_family(id: &str) -> Result<(), String> {
    validate_session_id(id)?;
    let metas = super::session_index::read_index().await?;
    for session_id in delete_targets(&metas, id) {
        let _ = super::session_tabs::remove_session_from_tabs(&session_id).await;
        let _ = super::session_store::delete_one(&session_id).await;
    }
    Ok(())
}

pub(crate) fn archive_targets(metas: &[AgentSessionMeta], id: &str) -> Vec<String> {
    family_ids(metas, id)
}

pub(crate) fn delete_targets(metas: &[AgentSessionMeta], id: &str) -> Vec<String> {
    let mut ids = family_ids(metas, id);
    ids.reverse();
    ids
}

pub(crate) fn restore_targets(
    metas: &[AgentSessionMeta],
    meta: &AgentSessionMeta,
) -> Result<Vec<String>, String> {
    let mut targets = ancestor_ids(metas, meta)?;
    targets.reverse();
    targets.push(meta.id.clone());
    Ok(targets)
}

pub(crate) fn family_ids(metas: &[AgentSessionMeta], id: &str) -> Vec<String> {
    let mut result = vec![id.to_string()];
    let mut queue = VecDeque::from([id.to_string()]);
    while let Some(parent) = queue.pop_front() {
        if result.len() >= MAX_FAMILY_SCAN {
            break;
        }
        for meta in metas {
            if parent_id(meta) != Some(parent.as_str())
                || result.iter().any(|seen| seen == &meta.id)
            {
                continue;
            }
            result.push(meta.id.clone());
            queue.push_back(meta.id.clone());
        }
    }
    result
}

fn ancestor_ids(
    metas: &[AgentSessionMeta],
    meta: &AgentSessionMeta,
) -> Result<Vec<String>, String> {
    let mut ancestors = Vec::new();
    let mut current = meta;
    while let Some(parent) = parent_id(current) {
        if ancestors.len() >= MAX_FAMILY_SCAN {
            return Err(generic_restore_error());
        }
        let Some(parent_meta) = metas.iter().find(|entry| entry.id == parent) else {
            return Err(generic_restore_error());
        };
        ancestors.push(parent_meta.id.clone());
        current = parent_meta;
    }
    Ok(ancestors)
}

fn parent_id(meta: &AgentSessionMeta) -> Option<&str> {
    meta.clone_parent_session_id
        .as_deref()
        .or(meta.parent_session_id.as_deref())
}

fn generic_restore_error() -> String {
    "Impossible de restaurer cette session.".to_string()
}

#[cfg(test)]
#[path = "session_family_tests.rs"]
mod tests;
