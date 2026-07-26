use super::subagent_registry::SubagentTerminalKind;
use super::types_session::AgentSession;
use std::future::Future;

// The caller holds the child lock; each terminal path then holds parent before registry.
pub(super) async fn complete_success<F, Fut>(
    parent_id: &str,
    child_id: &str,
    subagent_type: &str,
    child: &mut AgentSession,
    successful: bool,
    status: &str,
    summary: &str,
    after_report: F,
) -> Result<(), String>
where
    F: FnOnce(super::subagent_completion::TerminalOutcome) -> Fut,
    Fut: Future<Output = ()>,
{
    let parent_lock = super::session_store::lock_session(parent_id).await;
    let _parent_guard = parent_lock.lock().await;
    let mut parent = super::session_store::get(parent_id).await.ok();
    if !append_report(
        parent.as_mut(),
        child_id,
        &child.name,
        subagent_type,
        status,
        summary,
    )
    .await
    {
        let status_result = persist_failed_child(child).await;
        finish_registry(parent_id, child_id, false).await?;
        status_result?;
        return Err(super::subagent_completion::SUBAGENT_COMPLETION_ERROR.to_string());
    }
    after_report(super::subagent_completion::TerminalOutcome::new(
        successful,
        status,
        summary,
    ))
    .await;
    finish_registry(parent_id, child_id, true).await
}

pub(super) async fn complete_failure<F, Fut>(
    parent_id: &str,
    child_id: &str,
    subagent_type: &str,
    child: &mut AgentSession,
    append_generic_report: bool,
    generic_report_is_deliverable: bool,
    after_report: F,
) -> Result<(), String>
where
    F: FnOnce(super::subagent_completion::TerminalOutcome) -> Fut,
    Fut: Future<Output = ()>,
{
    let status_result = persist_failed_child(child).await;
    let parent_lock = super::session_store::lock_session(parent_id).await;
    let _parent_guard = parent_lock.lock().await;
    let mut parent = super::session_store::get(parent_id).await.ok();
    let report_persisted = append_generic_report
        && append_report(
            parent.as_mut(),
            child_id,
            &child.name,
            subagent_type,
            super::subagent_status::FAILED,
            super::subagent_completion::SUBAGENT_COMPLETION_ERROR,
        )
        .await;
    if report_persisted {
        after_report(super::subagent_completion::TerminalOutcome::failure()).await;
    }
    let report_is_ready =
        generic_report_is_deliverable && status_result.is_ok() && report_persisted;
    finish_registry(parent_id, child_id, report_is_ready).await?;
    status_result?;
    if append_generic_report && !report_persisted {
        return Err(super::subagent_completion::SUBAGENT_COMPLETION_ERROR.to_string());
    }
    Ok(())
}

pub(super) async fn complete_missing<F, Fut>(
    parent_id: &str,
    child_id: &str,
    subagent_type: &str,
    after_report: F,
) -> Result<(), String>
where
    F: FnOnce(super::subagent_completion::TerminalOutcome) -> Fut,
    Fut: Future<Output = ()>,
{
    let parent_lock = super::session_store::lock_session(parent_id).await;
    let _parent_guard = parent_lock.lock().await;
    let mut parent = super::session_store::get(parent_id).await.ok();
    let report_persisted = append_report(
        parent.as_mut(),
        child_id,
        "agent",
        subagent_type,
        super::subagent_status::FAILED,
        super::subagent_completion::SUBAGENT_COMPLETION_ERROR,
    )
    .await;
    if report_persisted {
        after_report(super::subagent_completion::TerminalOutcome::failure()).await;
    }
    finish_registry(parent_id, child_id, false).await
}

async fn append_report(
    parent: Option<&mut AgentSession>,
    child_id: &str,
    name: &str,
    subagent_type: &str,
    status: &str,
    summary: &str,
) -> bool {
    let Some(parent) = parent else {
        return false;
    };
    let report = super::subagent_hidden_reports::build_report(
        child_id.to_string(),
        name.to_string(),
        subagent_type.to_string(),
        status.to_string(),
        summary.to_string(),
    );
    super::subagent_hidden_reports::append_locked(parent, report)
        .await
        .is_ok()
}

async fn persist_failed_child(child: &mut AgentSession) -> Result<(), String> {
    super::subagent_task::finalize_loaded_session_after_run(
        child,
        super::subagent_status::FAILED,
        super::subagent_completion::SUBAGENT_COMPLETION_ERROR,
    );
    super::session_store::save(child)
        .await
        .map_err(|_| super::subagent_completion::SUBAGENT_COMPLETION_ERROR.to_string())
}

async fn finish_registry(
    parent_id: &str,
    child_id: &str,
    report_persisted: bool,
) -> Result<(), String> {
    let kind = if report_persisted {
        SubagentTerminalKind::ReportPersisted
    } else {
        SubagentTerminalKind::ReportPersistenceFailed
    };
    super::subagent_registry::complete_child_or_parent(parent_id, child_id, kind).await
}
