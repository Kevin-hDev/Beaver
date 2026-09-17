#[derive(Debug, Eq, PartialEq)]
pub enum ArchiveOutcome {
    Archived,
    Active,
    NotFound,
}

pub async fn archive(child_id: &str) -> Result<ArchiveOutcome, String> {
    archive_inner(child_id, None).await
}

pub async fn archive_owned(
    child_id: &str,
    parent_id: &str,
) -> Result<ArchiveOutcome, String> {
    archive_inner(child_id, Some(parent_id)).await
}

async fn archive_inner(
    child_id: &str,
    expected_parent: Option<&str>,
) -> Result<ArchiveOutcome, String> {
    super::session_store::validate_session_id(child_id)?;
    let lock = super::session_store::lock_session(child_id).await;
    let guard = lock.lock().await;
    let mut child = match super::session_store::get(child_id).await {
        Ok(child) => child,
        Err(_) => return Ok(ArchiveOutcome::NotFound),
    };
    if child.parent_session_id.is_none()
        || expected_parent.is_some_and(|parent| child.parent_session_id.as_deref() != Some(parent))
    {
        return Ok(ArchiveOutcome::NotFound);
    }
    if super::subagent_live_state::has_pending_work(&child).await {
        return Ok(ArchiveOutcome::Active);
    }
    if child.archived_at.is_none() {
        child.archived_at = Some(chrono::Utc::now());
        super::session_store::save(&child).await?;
    }
    drop(guard);
    archive_descendants(child_id).await;
    Ok(ArchiveOutcome::Archived)
}

async fn archive_descendants(child_id: &str) {
    let Ok(metas) = super::session_index::read_index().await else {
        return;
    };
    for descendant in super::session_family::archive_targets(&metas, child_id)
        .into_iter()
        .filter(|id| id != child_id)
    {
        let _ = super::session_archive::archive(&descendant).await;
    }
}
