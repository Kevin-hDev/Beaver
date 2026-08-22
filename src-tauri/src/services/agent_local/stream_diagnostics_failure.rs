use chrono::Utc;

use super::stream_diagnostics_support as support;
use super::types_diagnostics::{
    AgentDiagnosticRun, AgentErrorDiagnosticSummary, AgentStreamFailure,
};
use super::types_session::AgentSession;

pub(crate) fn push_failure(session: &mut AgentSession, message: &str, is_connection: bool) {
    let active = session
        .active_todo_run_id
        .as_ref()
        .and_then(|id| session.todo_runs.iter().find(|run| &run.id == id));
    session.stream_failures.push(AgentStreamFailure {
        code: safe_code(message),
        occurred_at: Utc::now(),
        is_connection,
        active_todo_run_id: active.map(|run| run.id.clone()),
        active_todo_title: active.map(|run| support::clip(&run.title)),
    });
    support::trim(&mut session.stream_failures, support::MAX_STREAM_FAILURES);
}

pub(crate) fn apply_failure(
    session: &mut AgentSession,
    idx: usize,
    message: &str,
    is_connection: bool,
) {
    let todo = support::active_todo(session);
    let run = &mut session.diagnostic_runs[idx];
    let error_type = classify_error(message, is_connection);
    run.status = "failed".to_string();
    run.severity = "error".to_string();
    run.error_type = Some(error_type.clone());
    run.ended_at = Some(Utc::now());
    run.updated_at = Utc::now();
    run.active_todo = todo;
    run.safe_summary = Some(safe_summary(run, &error_type, message));
    support::push_event(run, "failed", message, None, Some(&error_type));
}

pub(crate) fn summary_from_run(run: &AgentDiagnosticRun) -> AgentErrorDiagnosticSummary {
    AgentErrorDiagnosticSummary {
        request_id: run.request_id.clone(),
        phase: run.phase.clone(),
        error_type: run
            .error_type
            .clone()
            .unwrap_or_else(|| "unknown".to_string()),
        last_tool_name: run.last_tool.as_ref().map(|tool| tool.name.clone()),
        safe_summary: run
            .safe_summary
            .clone()
            .unwrap_or_else(|| "Flux interrompu.".to_string()),
    }
}

fn safe_summary(run: &AgentDiagnosticRun, error_type: &str, message: &str) -> String {
    if message
        .to_ascii_lowercase()
        .contains("plan mode workflow could not be enforced")
    {
        return "Workflow Plan Mode non respecté par le modèle.".to_string();
    }
    if let Some(tool) = &run.last_tool {
        if error_type == "max_turns" {
            return support::clip(&format!(
                "Limite de tours agent atteinte après le dernier tool {}.",
                tool.name
            ));
        }
        if run.phase != "tool_execution" || tool.status == "completed" {
            return support::clip(&format!(
                "Interruption après le dernier tool {} ({error_type}).",
                tool.name
            ));
        }
        return support::clip(&format!(
            "Interruption pendant le tool {} ({error_type}).",
            tool.name
        ));
    }
    support::clip(&format!(
        "Interruption pendant {} ({error_type}).",
        run.phase
    ))
}

fn safe_code(message: &str) -> String {
    let code = classify_error(message, is_connection_error(message));
    if code == "unknown" {
        "stream_error".to_string()
    } else {
        code
    }
}

pub(super) fn classify_error(message: &str, is_connection: bool) -> String {
    let lower = message.to_ascii_lowercase();
    if matches!(
        lower.as_str(),
        "rate_limit"
            | "auth_failed"
            | "oauth_reauthentication_required"
            | "provider_access_unavailable"
            | "provider_quota_exhausted"
            | "provider_payload_too_large"
            | "provider_configuration_invalid"
    ) {
        return lower;
    }
    if is_connection || is_connection_error(message) {
        return "connection_lost".to_string();
    }
    if lower.contains("timeout") {
        return "timeout".to_string();
    }
    if lower.contains("overloaded") || lower.contains("temporarily unavailable") {
        return "provider_overloaded".to_string();
    }
    if lower == "provider_temporarily_unavailable" {
        return "provider_overloaded".to_string();
    }
    if matches!(
        lower.as_str(),
        "provider_connection_failed" | "provider_request_rejected"
    ) {
        return "provider_error".to_string();
    }
    if message.contains("Limite de tours") {
        return "max_turns".to_string();
    }
    if message.contains("répété") || message.contains("circuit") {
        return "circuit_breaker".to_string();
    }
    if lower.contains("http")
        || lower.contains("api")
        || lower.contains("rate")
        || lower.contains("auth")
        || lower.contains("internal server error")
        || lower.contains("bad request")
        || lower.contains("sse:")
    {
        return "provider_error".to_string();
    }
    if lower.contains("plan mode workflow") {
        return "tool_error".to_string();
    }
    "unknown".to_string()
}

pub(crate) fn is_connection_error(message: &str) -> bool {
    message
        .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
        .any(|part| matches!(part, "ollama_connection_lost" | "provider_connection_failed"))
}

#[cfg(test)]
#[path = "stream_diagnostics_failure_tests.rs"]
mod tests;
