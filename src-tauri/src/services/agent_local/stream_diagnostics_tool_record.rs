use super::diagnostic_redaction;
use super::stream_diagnostics_support as support;
use super::stream_diagnostics_tools as diagnostic_tools;
use super::tool_result_contract::ToolResultStatus;
use super::types_diagnostics::AgentDiagnosticTool;
use super::types_tools::ToolResult;

pub async fn record_tool(
    session_id: &str,
    request_id: &str,
    name: &str,
    status: &str,
    args: Option<serde_json::Value>,
    result: Option<&ToolResult>,
) {
    let is_error = result.is_some_and(|value| value.is_error);
    let result_status = result.map(|value| value.status);
    let error_code = result
        .and_then(|value| value.error.as_ref())
        .map(|error| error.code.to_string());
    let message = message(name, status, result_status, error_code.as_deref());

    let _ = support::update_run(session_id, request_id, |session, run| {
        let phase = if status == "completed" {
            "tool_result"
        } else {
            "tool_execution"
        };
        run.phase = phase.to_string();
        run.severity = severity(result).to_string();
        let tool = AgentDiagnosticTool {
            name: support::clip(name),
            status: status.to_string(),
            args: args.clone().map(diagnostic_redaction::redact_value),
            is_error,
            result_status,
            error_code: error_code.clone(),
            error_category: result
                .and_then(|value| value.error.as_ref())
                .map(|error| error.category),
            retryable: result
                .and_then(|value| value.error.as_ref())
                .map(|error| error.retryable),
            truncated: result.is_some_and(|value| value.truncated),
            warning_count: result.map_or(0, |value| value.warnings.len()),
        };
        run.last_tool = Some(tool.clone());
        run.recent_tools.push(tool);
        support::trim(
            &mut run.recent_tools,
            diagnostic_tools::MAX_DIAGNOSTIC_TOOLS,
        );
        run.active_todo = support::active_todo(session);
        run.safe_summary = Some(support::clip(&message));
        support::push_event(run, phase, &message, Some(name), error_code.as_deref());
    })
    .await;
}

fn message(
    name: &str,
    status: &str,
    result_status: Option<ToolResultStatus>,
    error_code: Option<&str>,
) -> String {
    match (result_status, error_code) {
        (Some(result_status), Some(code)) => {
            format!("Tool {name} {status} ({}, {code})", result_status.as_str())
        }
        (Some(result_status), None) => {
            format!("Tool {name} {status} ({})", result_status.as_str())
        }
        (None, _) => format!("Tool {name} {status}"),
    }
}

fn severity(result: Option<&ToolResult>) -> &'static str {
    if result.is_some_and(|value| {
        value.is_error
            || value.status == ToolResultStatus::Partial
            || value.truncated
            || !value.warnings.is_empty()
    }) {
        "warning"
    } else {
        "info"
    }
}

#[cfg(test)]
mod tests {
    use super::severity;
    use crate::services::agent_local::types_tools::ToolResult;

    #[test]
    fn partial_and_error_results_raise_diagnostic_severity() {
        let success = ToolResult::ok("ok");
        let partial = ToolResult::partial("partial", ["warning"]);
        let error = ToolResult::execution("test_failure", "failed", false);

        assert_eq!(severity(Some(&success)), "info");
        assert_eq!(severity(Some(&partial)), "warning");
        assert_eq!(severity(Some(&error)), "warning");
    }
}
