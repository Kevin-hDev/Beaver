pub(super) struct FinalizedSubagent {
    pub(super) queued_followup: bool,
    pub(super) session_status: String,
}

pub fn effective_session_status(status: &str, queued_followup: bool) -> &str {
    if queued_followup {
        super::subagent_status::RUNNING
    } else {
        status
    }
}

pub(super) fn should_continue_same_run(status: &str, has_queued_prompt: bool) -> bool {
    status == super::subagent_status::COMPLETED && has_queued_prompt
}

pub(super) fn finalize_loaded_session_after_run(
    session: &mut AgentSession,
    status: &str,
    summary: &str,
) -> FinalizedSubagent {
    let finalized = apply_finalized_subagent_state(session, status, summary);
    session.subagent_last_activity = Some(super::types_session::SubagentLastActivity {
        kind: "status".to_string(),
        label: final_activity_label(&finalized.session_status).to_string(),
        detail: Some(summary.chars().take(220).collect()),
        updated_at: chrono::Utc::now(),
    });
    finalized
}

pub(super) fn apply_finalized_subagent_state(
    session: &mut AgentSession,
    status: &str,
    summary: &str,
) -> FinalizedSubagent {
    let queued_followup =
        should_continue_same_run(status, !session.subagent_queued_prompts.is_empty());
    let session_status = effective_session_status(status, queued_followup);
    if status == super::subagent_status::CANCELLED {
        session.subagent_queued_prompts.clear();
    }
    session.subagent_summary = Some(summary.to_string());
    session.subagent_status = Some(session_status.to_string());
    session.updated_at = Some(chrono::Utc::now());
    FinalizedSubagent {
        queued_followup,
        session_status: session_status.to_string(),
    }
}

pub(super) fn final_activity_label(status: &str) -> &'static str {
    match status {
        super::subagent_status::RUNNING => "En cours",
        super::subagent_status::CANCELLED => "Annulé",
        super::subagent_status::FAILED => "Échoué",
        _ => "Terminé",
    }
}
