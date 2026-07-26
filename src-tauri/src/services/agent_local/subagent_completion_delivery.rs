#[cfg(test)]
pub(super) async fn persist_instruction_delivery_failure(
    parent_session_id: &str,
    child_session_id: &str,
    subagent_type: &str,
) -> Result<(), String> {
    persist_instruction_delivery_failure_inner(
        parent_session_id,
        child_session_id,
        subagent_type,
        None,
        |_| async {},
    )
    .await
    .map(|_| ())
}

pub(super) async fn persist_instruction_delivery_failure_inner<F, Fut>(
    parent_session_id: &str,
    child_session_id: &str,
    subagent_type: &str,
    expected_owner: Option<(&str, &str)>,
    after_report: F,
) -> Result<bool, String>
where
    F: FnOnce(super::subagent_completion::TerminalOutcome) -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    let lock = super::session_store::lock_session(child_session_id).await;
    let _guard = lock.lock().await;
    let mut child = match super::session_store::get(child_session_id).await {
        Ok(child) => child,
        Err(_) => {
            if !super::subagent_completion_ownership::missing(child_session_id, expected_owner)
                .await
            {
                return Ok(false);
            }
            super::subagent_completion_boundary::complete_missing(
                parent_session_id,
                child_session_id,
                subagent_type,
                after_report,
            )
            .await?;
            return Err(SUBAGENT_COMPLETION_ERROR.to_string());
        }
    };
    if !super::subagent_completion_ownership::loaded(&child, expected_owner).await {
        return Ok(false);
    }
    super::subagent_completion_boundary::complete_failure(
        parent_session_id,
        child_session_id,
        subagent_type,
        &mut child,
        true,
        true,
        after_report,
    )
    .await?;
    Ok(true)
}
