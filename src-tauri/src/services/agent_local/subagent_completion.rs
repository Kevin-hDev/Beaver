use super::subagent_task::FinalizedSubagent;

pub(super) const SUBAGENT_COMPLETION_ERROR: &str =
    "Le sous-agent n'a pas pu finaliser son rapport.";

#[derive(Debug, Eq, PartialEq)]
pub(super) struct TerminalOutcome {
    pub success: bool,
    pub status: String,
    pub summary: String,
}

impl TerminalOutcome {
    pub fn new(success: bool, status: &str, summary: &str) -> Self {
        Self {
            success,
            status: status.to_string(),
            summary: summary.to_string(),
        }
    }

    pub fn failure() -> Self {
        Self::new(false, super::subagent_status::FAILED, SUBAGENT_COMPLETION_ERROR)
    }
}

#[cfg(test)]
pub(super) async fn persist_terminal_completion(
    parent_session_id: &str,
    child_session_id: &str,
    subagent_type: &str,
    status: &str,
    summary: &str,
) -> Result<FinalizedSubagent, String> {
    persist_terminal_completion_inner(
        parent_session_id,
        child_session_id,
        subagent_type,
        status,
        summary,
        status == super::subagent_status::COMPLETED,
        None,
        || async {},
        |_| async {},
    )
    .await?
    .ok_or_else(|| SUBAGENT_COMPLETION_ERROR.to_string())
}

include!("subagent_completion_delivery.rs");

#[cfg(test)]
pub(super) async fn persist_terminal_completion_with_after_report<F, Fut>(
    parent_session_id: &str,
    child_session_id: &str,
    subagent_type: &str,
    status: &str,
    summary: &str,
    after_report: F,
) -> Result<FinalizedSubagent, String>
where
    F: FnOnce(TerminalOutcome) -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    persist_terminal_completion_inner(
        parent_session_id,
        child_session_id,
        subagent_type,
        status,
        summary,
        status == super::subagent_status::COMPLETED,
        None,
        || async {},
        after_report,
    )
    .await?
    .ok_or_else(|| SUBAGENT_COMPLETION_ERROR.to_string())
}

#[cfg(test)]
pub(super) async fn persist_terminal_completion_with_hooks<FL, FLFut, FR, FRFut>(
    parent_session_id: &str,
    child_session_id: &str,
    subagent_type: &str,
    status: &str,
    summary: &str,
    after_child_loaded: FL,
    after_report: FR,
) -> Result<FinalizedSubagent, String>
where
    FL: FnOnce() -> FLFut,
    FLFut: std::future::Future<Output = ()>,
    FR: FnOnce(TerminalOutcome) -> FRFut,
    FRFut: std::future::Future<Output = ()>,
{
    persist_terminal_completion_inner(
        parent_session_id,
        child_session_id,
        subagent_type,
        status,
        summary,
        status == super::subagent_status::COMPLETED,
        None,
        after_child_loaded,
        after_report,
    )
    .await?
    .ok_or_else(|| SUBAGENT_COMPLETION_ERROR.to_string())
}

pub(super) async fn persist_terminal_completion_inner<FL, FLFut, FR, FRFut>(
    parent_session_id: &str,
    child_session_id: &str,
    subagent_type: &str,
    status: &str,
    summary: &str,
    successful: bool,
    expected_owner: Option<(&str, &str)>,
    after_child_loaded: FL,
    after_report: FR,
) -> Result<Option<FinalizedSubagent>, String>
where
    FL: FnOnce() -> FLFut,
    FLFut: std::future::Future<Output = ()>,
    FR: FnOnce(TerminalOutcome) -> FRFut,
    FRFut: std::future::Future<Output = ()>,
{
    let lock = super::session_store::lock_session(child_session_id).await;
    let _guard = lock.lock().await;
    let mut child = match super::session_store::get(child_session_id).await {
        Ok(child) => child,
        Err(_) => {
            if !super::subagent_completion_ownership::missing(child_session_id, expected_owner)
                .await
            {
                return Ok(None);
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
        return Ok(None);
    }
    after_child_loaded().await;
    let finalized =
        super::subagent_task::finalize_loaded_session_after_run(&mut child, status, summary);
    if super::session_store::save(&child).await.is_err() {
        super::subagent_completion_boundary::complete_failure(
            parent_session_id,
            child_session_id,
            subagent_type,
            &mut child,
            true,
            false,
            after_report,
        )
        .await?;
        return Err(SUBAGENT_COMPLETION_ERROR.to_string());
    }

    if finalized.queued_followup {
        return Ok(Some(finalized));
    }

    super::subagent_completion_boundary::complete_success(
        parent_session_id,
        child_session_id,
        subagent_type,
        &mut child,
        successful,
        &finalized.session_status,
        summary,
        after_report,
    )
    .await?;
    Ok(Some(finalized))
}
