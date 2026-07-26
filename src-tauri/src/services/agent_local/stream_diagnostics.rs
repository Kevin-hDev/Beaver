use chrono::Utc;
use serde_json::json;
use uuid::Uuid;

use super::diagnostic_redaction;
use super::stream_diagnostics_failure as failure;
use super::stream_diagnostics_support as support;
use super::stream_diagnostics_tools as diagnostic_tools;
use super::types_diagnostics::{
    AgentDiagnosticRun, AgentDiagnosticTool, AgentErrorDiagnosticSummary,
};
use super::types_session::AgentSession;

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

pub async fn record_tool(
    session_id: &str,
    request_id: &str,
    name: &str,
    status: &str,
    args: Option<serde_json::Value>,
    is_error: bool,
) {
    let message = format!("Tool {name} {status}");
    let _ = support::update_run(session_id, request_id, |session, run| {
        let phase = if status == "completed" {
            "tool_result"
        } else {
            "tool_execution"
        };
        run.phase = phase.to_string();
        run.severity = if is_error { "warning" } else { "info" }.to_string();
        let tool = AgentDiagnosticTool {
            name: support::clip(name),
            status: status.to_string(),
            args: args.clone().map(diagnostic_redaction::redact_value),
            is_error,
        };
        run.last_tool = Some(tool.clone());
        run.recent_tools.push(tool);
        support::trim(
            &mut run.recent_tools,
            diagnostic_tools::MAX_DIAGNOSTIC_TOOLS,
        );
        run.active_todo = support::active_todo(session);
        run.safe_summary = Some(support::clip(&message));
        support::push_event(run, phase, &message, Some(name), None);
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

pub async fn diagnostics_text(session_id: &str, limit: usize) -> Result<String, String> {
    super::session_store::validate_session_id(session_id)?;
    let session = super::session_store::get(session_id).await?;
    let limit = diagnostic_tools::bounded_tool_limit(limit);
    let recent_tools = diagnostic_tools::recent_relevant_tools(&session, limit);
    let recent_work_tools = diagnostic_tools::recent_work_tools(&session, limit);
    serde_json::to_string_pretty(&json!({
        "latest": session.diagnostic_runs.last(),
        "current_tool": session.diagnostic_runs.last().and_then(|run| run.last_tool.as_ref()),
        "last_relevant_tool": recent_tools.first(),
        "last_work_tool": recent_work_tools.first(),
        "recent_tools": recent_tools,
        "recent_work_tools": recent_work_tools,
        "recent": session.diagnostic_runs.iter().rev().take(5).collect::<Vec<_>>(),
        "legacy_stream_failures": session.stream_failures.iter().rev().take(5).collect::<Vec<_>>(),
    }))
    .map_err(|_| "Diagnostics indisponibles.".to_string())
}

pub(crate) fn push_failure(session: &mut AgentSession, message: &str, is_connection: bool) {
    failure::push_failure(session, message, is_connection);
}
