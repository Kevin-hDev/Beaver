use super::types_session::SubagentHiddenReport;
use chrono::Utc;
use std::collections::BTreeSet;
use uuid::Uuid;

pub use super::subagent_report_context::append_context;
#[cfg(test)]
pub use super::subagent_report_context::{
    ensure_report_policy, report_to_message, SUBAGENT_REPORT_CONTEXT_PREFIX,
    SUBAGENT_REPORT_POLICY_PREFIX,
};

pub(super) const MAX_PENDING_REPORTS: usize = 16;
const MAX_REPORT_SUMMARY_CHARS: usize = 12_000;

#[cfg(test)]
pub async fn append(parent_id: &str, report: SubagentHiddenReport) -> Result<(), String> {
    let lock = super::session_store::lock_session(parent_id).await;
    let _guard = lock.lock().await;
    let mut session = super::session_store::get(parent_id).await?;
    append_locked(&mut session, report).await
}

pub(super) async fn append_locked(
    session: &mut super::types_session::AgentSession,
    report: SubagentHiddenReport,
) -> Result<(), String> {
    if session
        .subagent_hidden_reports
        .iter()
        .any(|seen| is_same_report(seen, &report))
    {
        return Ok(());
    }
    if session.subagent_hidden_reports.len() >= MAX_PENDING_REPORTS {
        return Err("La file de rapports de sous-agents est pleine.".to_string());
    }
    session.subagent_hidden_reports.push(report);
    super::session_store::save(session).await
}

pub async fn peek_reports(session_id: &str) -> Vec<SubagentHiddenReport> {
    let lock = super::session_store::lock_session(session_id).await;
    let _guard = lock.lock().await;
    super::session_store::get(session_id)
        .await
        .map(|session| session.subagent_hidden_reports)
        .unwrap_or_default()
}

#[cfg(test)]
pub async fn acknowledge_reports(session_id: &str, report_ids: &[String]) -> Result<(), String> {
    let cancel = tokio_util::sync::CancellationToken::new();
    acknowledge_reports_cancellable(session_id, report_ids, &cancel).await
}

pub async fn acknowledge_reports_cancellable(
    session_id: &str,
    report_ids: &[String],
    cancel: &tokio_util::sync::CancellationToken,
) -> Result<(), String> {
    if cancel.is_cancelled() {
        return Err("Annulé".to_string());
    }
    let lock = super::session_store::lock_session(session_id).await;
    let _guard = lock.lock().await;
    let mut session = super::session_store::get(session_id).await?;
    let report_ids = report_ids.iter().collect::<BTreeSet<_>>();
    session
        .subagent_hidden_reports
        .retain(|report| !report_ids.contains(&report.id));
    if cancel.is_cancelled() {
        return Err("Annulé".to_string());
    }
    super::session_store::save(&session).await
}

pub async fn has_pending_except(session_id: &str, ignored_child_ids: &BTreeSet<String>) -> bool {
    let lock = super::session_store::lock_session(session_id).await;
    let _guard = lock.lock().await;
    super::session_store::get(session_id)
        .await
        .map(|session| {
            session
                .subagent_hidden_reports
                .iter()
                .any(|report| !ignored_child_ids.contains(&report.child_session_id))
        })
        .unwrap_or(false)
}

pub fn build_report(
    child_session_id: String,
    name: String,
    subagent_type: String,
    status: String,
    summary: String,
) -> SubagentHiddenReport {
    SubagentHiddenReport {
        id: Uuid::new_v4().to_string(),
        child_session_id,
        name,
        subagent_type,
        status,
        summary: truncate_chars(&summary, MAX_REPORT_SUMMARY_CHARS),
        created_at: Utc::now(),
    }
}

fn is_same_report(seen: &SubagentHiddenReport, report: &SubagentHiddenReport) -> bool {
    seen.id == report.id
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    let mut output = value.chars().take(max_chars).collect::<String>();
    if value.chars().count() > max_chars {
        output.push_str("\n[rapport tronqué]");
    }
    output
}

#[cfg(test)]
#[path = "subagent_hidden_reports_tests.rs"]
mod tests;
