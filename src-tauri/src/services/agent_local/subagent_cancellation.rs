pub async fn cancel(child_id: &str) -> Result<bool, String> {
    cancel_inner(child_id, None).await
}

pub async fn cancel_owned(child_id: &str, parent_id: &str) -> Result<bool, String> {
    cancel_inner(child_id, Some(parent_id)).await
}

async fn cancel_inner(child_id: &str, expected_parent: Option<&str>) -> Result<bool, String> {
    super::session_store::validate_session_id(child_id)?;
    let lock = super::session_store::lock_session(child_id).await;
    let _guard = lock.lock().await;
    let mut child = match super::session_store::get(child_id).await {
        Ok(child) => child,
        Err(_) => return Ok(false),
    };
    if expected_parent.is_some_and(|parent| child.parent_session_id.as_deref() != Some(parent)) {
        return Ok(false);
    }
    let Some(active) = super::subagent_registry::active_run_for_child(child_id).await else {
        return Ok(false);
    };
    if active.cancelled || child.subagent_run_id.as_deref() != Some(&active.run_id) {
        return Ok(false);
    }
    if !super::subagent_registry::cancel_execution(child_id, &active.execution_id).await {
        return Ok(false);
    }
    child.subagent_status = Some(super::subagent_status::CANCELLED.to_string());
    child.subagent_queued_prompts.clear();
    child.updated_at = Some(chrono::Utc::now());
    super::session_store::save(&child).await?;
    Ok(true)
}
