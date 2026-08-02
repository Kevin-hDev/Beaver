use chrono::Utc;
use uuid::Uuid;

use super::stream_diagnostics_failure as failure;
use super::stream_diagnostics_support as support;
use super::types_diagnostics::{AgentDiagnosticRun, AgentErrorDiagnosticSummary};
use super::types_session::AgentSession;

pub use super::stream_diagnostics_tool_record::record_tool;

pub async fn start_request(session_id: &str, generation: u64) -> String {
    let request_id = Uuid::new_v4().to_string();
    let _ = support::update_session(session_id, |session| {
        let now = Utc::now();
        session.diagnostic_runs.push(AgentDiagnosticRun {
            request_id: request_id.clone(),
            generation,
            status: "running".to_string(),
            severity: "info".to_string(),
            started_at: now,
            updated_at: now,
            ended_at: None,
            phase: "request_start".to_string(),
            error_type: None,
            last_tool: None,
            recent_tools: vec![],
            active_todo: support::active_todo(session),
            safe_summary: Some("Requête agent démarrée.".to_string()),
            events: vec![support::event(
                "request_start",
                "Requête agent démarrée.",
                None,
                None,
            )],
        });
        support::trim(&mut session.diagnostic_runs, support::MAX_DIAGNOSTIC_RUNS);
    })
    .await;
    request_id
}

pub async fn mark_phase(session_id: &str, request_id: &str, phase: &str, message: &str) {
    let _ = support::update_run(session_id, request_id, |session, run| {
        run.phase = phase.to_string();
        run.safe_summary = Some(support::clip(message));
        run.active_todo = support::active_todo(session);
        support::push_event(run, phase, message, None, None);
    })
    .await;
}

pub async fn record_retry(session_id: &str, request_id: &str, message: &str) {
    let _ = support::update_run(session_id, request_id, |_session, run| {
        run.phase = "retrying".to_string();
        run.severity = "warning".to_string();
        run.safe_summary = Some(support::clip(message));
        support::push_event(run, "retrying", message, None, None);
    })
    .await;
}

pub async fn record_extension_tools(
    session_id: &str,
    request_id: &str,
    phase: &str,
    names: &[String],
) {
    let total = names.len();
    let max_items = super::provider_tool_limits::MAX_CAPACITY_DIAGNOSTIC_ITEMS;
    let names = names
        .iter()
        .take(max_items)
        .cloned()
        .collect::<Vec<_>>()
        .join(", ");
    let suffix = if total > max_items {
        format!(" (+{} more)", total - max_items)
    } else {
        String::new()
    };
    let label = if phase == "extension_plugins_omitted" {
        "Extension plugins omitted by provider capacity"
    } else {
        "Extension tools selected"
    };
    let message = format!("{label}: {names}{suffix}");
    let _ = support::update_run(session_id, request_id, |_session, run| {
        support::push_event(run, phase, &message, None, None);
    })
    .await;
}

pub async fn record_completed(session_id: &str, request_id: &str) {
    let _ = support::update_run(session_id, request_id, |session, run| {
        run.status = "completed".to_string();
        run.phase = "completed".to_string();
        run.severity = "info".to_string();
        run.ended_at = Some(Utc::now());
        run.active_todo = support::active_todo(session);
        run.safe_summary = Some("Requête terminée.".to_string());
        support::push_event(run, "completed", "Requête terminée.", None, None);
    })
    .await;
}

pub async fn record_cancelled(session_id: &str, request_id: &str) {
    let _ = support::update_run(session_id, request_id, |_session, run| {
        run.status = "cancelled".to_string();
        run.phase = "failed".to_string();
        run.severity = "warning".to_string();
        run.error_type = Some("cancelled".to_string());
        run.ended_at = Some(Utc::now());
        run.safe_summary = Some("Requête annulée.".to_string());
        support::push_event(run, "failed", "Requête annulée.", None, Some("cancelled"));
    })
    .await;
}

pub async fn record_failure(
    session_id: &str,
    request_id: Option<&str>,
    message: &str,
    is_connection: bool,
) -> Option<AgentErrorDiagnosticSummary> {
    let mut summary = None;
    let _ = support::update_session(session_id, |session| {
        push_failure(session, message, is_connection);
        if let Some(id) = request_id {
            if let Some(idx) = support::find_run(session, id) {
                failure::apply_failure(session, idx, message, is_connection);
                summary = Some(failure::summary_from_run(&session.diagnostic_runs[idx]));
            }
        }
    })
    .await;
    summary
}

pub(crate) fn push_failure(session: &mut AgentSession, message: &str, is_connection: bool) {
    failure::push_failure(session, message, is_connection);
}
