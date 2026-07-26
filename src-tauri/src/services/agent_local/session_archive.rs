use chrono::Utc;

use super::session_store::{get, lock_session, save, validate_session_id};
use super::types_session::AgentSessionMeta;

pub async fn list_archived() -> Result<Vec<AgentSessionMeta>, String> {
    let mut metas = crate::services::agent_local::session_index::read_index().await?;
    metas.retain(|m| m.archived_at.is_some());
    sort_recent_first(&mut metas);
    Ok(metas)
}

pub async fn archive(id: &str) -> Result<(), String> {
    validate_session_id(id)?;
    let lock = lock_session(id).await;
    let _guard = lock.lock().await;
    let mut session = get(id).await?;
    if session.archived_at.is_none() {
        session.archived_at = Some(Utc::now());
    }
    save(&session).await
}

pub async fn restore(id: &str) -> Result<(), String> {
    validate_session_id(id)?;
    let lock = lock_session(id).await;
    let _guard = lock.lock().await;
    let mut session = get(id).await?;
    session.archived_at = None;
    save(&session).await
}

pub(crate) fn is_active(meta: &AgentSessionMeta) -> bool {
    meta.archived_at.is_none()
}

pub(crate) fn activity_at(meta: &AgentSessionMeta) -> chrono::DateTime<Utc> {
    meta.updated_at.unwrap_or(meta.created_at)
}

pub(crate) fn sort_recent_first(metas: &mut [AgentSessionMeta]) {
    metas.sort_by_key(|m| std::cmp::Reverse(activity_at(m)));
}
