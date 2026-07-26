use std::collections::BTreeSet;

pub(super) async fn ensure(
    parent_session_id: &str,
    report_delivery: &mut super::subagent_report_delivery::SubagentReportDelivery,
) -> Result<(), String> {
    report_delivery.refresh_terminal_signal().await;
    if report_delivery.persistence_failed() {
        return Err(super::subagent_completion::SUBAGENT_COMPLETION_ERROR.to_string());
    }
    let session = super::session_store::get(parent_session_id)
        .await
        .map_err(|_| super::subagent_completion::SUBAGENT_COMPLETION_ERROR.to_string())?;
    super::subagent_instruction_delivery::validate_persisted_queue(
        &session.subagent_queued_prompts,
    )?;
    let pending_correction = !session.subagent_queued_prompts.is_empty();
    let pending_report = super::subagent_hidden_reports::has_pending_except(
        parent_session_id,
        &BTreeSet::new(),
    )
    .await;
    let snapshot = super::subagent_registry::parent_snapshot(parent_session_id).await;
    if pending_correction || pending_report || !snapshot.active_child_ids.is_empty() {
        return Err(super::agent_loop_errors::max_turns_message());
    }
    if snapshot
        .terminal_state
        .is_some_and(|state| state.sequence > 0)
    {
        return Err(super::subagent_completion::SUBAGENT_COMPLETION_ERROR.to_string());
    }
    Ok(())
}
