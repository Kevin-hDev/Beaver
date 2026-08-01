use chrono::Utc;

use super::*;
use crate::services::agent_local::types_diagnostics::{
    AgentDiagnosticRun, AgentDiagnosticTool,
};

fn run(phase: &str, tool_status: &str) -> AgentDiagnosticRun {
    AgentDiagnosticRun {
        request_id: "request".to_string(),
        generation: 1,
        status: "running".to_string(),
        severity: "info".to_string(),
        started_at: Utc::now(),
        updated_at: Utc::now(),
        ended_at: None,
        phase: phase.to_string(),
        error_type: None,
        last_tool: Some(AgentDiagnosticTool {
            name: "list_dir".to_string(),
            status: tool_status.to_string(),
            args: None,
            is_error: false,
        }),
        recent_tools: Vec::new(),
        active_todo: None,
        safe_summary: None,
        events: Vec::new(),
    }
}

#[test]
fn provider_failure_after_completed_tool_is_not_attributed_to_tool() {
    let summary = safe_summary(&run("model_stream", "completed"), "provider_error", "error");

    assert_eq!(
        summary,
        "Interruption après le dernier tool list_dir (provider_error)."
    );
}

#[test]
fn running_tool_failure_keeps_during_wording() {
    let summary = safe_summary(&run("tool_execution", "started"), "tool_error", "error");

    assert_eq!(
        summary,
        "Interruption pendant le tool list_dir (tool_error)."
    );
}

#[test]
fn provider_transport_failure_is_classified_as_a_connection_loss() {
    assert!(is_connection_error("provider_connection_failed"));
    assert!(is_connection_error(
        "Compression Ollama : ollama_connection_lost"
    ));
    assert_eq!(
        classify_error("provider_connection_failed", false),
        "connection_lost"
    );
    assert_eq!(safe_code("provider_connection_failed"), "connection_lost");
}

#[test]
fn provider_rejection_is_not_classified_as_a_connection_loss() {
    assert!(!is_connection_error("provider_request_rejected"));
    assert_eq!(
        classify_error("provider_request_rejected", false),
        "provider_error"
    );
}
