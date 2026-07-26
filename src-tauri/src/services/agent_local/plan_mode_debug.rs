use super::plan_mode_controller::PlanModeDecision;
use super::types_ollama::StreamResult;
use super::types_plan::AgentPlanWorkflowStatus;

pub fn controller_decision(
    session_id: &str,
    workflow: AgentPlanWorkflowStatus,
    repair_count: usize,
    result: &StreamResult,
    decision: &PlanModeDecision,
) {
    let decision_label = match decision {
        PlanModeDecision::Accept => "accept",
        PlanModeDecision::Retry(_) => "retry",
        PlanModeDecision::Fail(_) => "fail",
    };
    eprintln!(
        "[plan-mode] session={} workflow={workflow:?} repairs={repair_count} content_chars={} question={} tools=[{}] decision={decision_label}",
        short_id(session_id),
        result.content.chars().count(),
        has_question(&result.content),
        tool_names(result),
    );
}

pub fn workflow_failed(session_id: &str, request_id: &str, message: &str) {
    eprintln!(
        "[plan-mode] failed session={} request={} reason={}",
        short_id(session_id),
        short_id(request_id),
        sanitize_reason(message),
    );
}

fn tool_names(result: &StreamResult) -> String {
    result
        .tool_calls
        .iter()
        .take(8)
        .map(|(name, _)| name.as_str())
        .collect::<Vec<_>>()
        .join(",")
}

fn short_id(value: &str) -> &str {
    value.get(..8).unwrap_or(value)
}

fn has_question(content: &str) -> bool {
    content.contains('?') || content.contains('？')
}

fn sanitize_reason(message: &str) -> String {
    message
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || ch.is_ascii_whitespace())
        .take(120)
        .collect()
}
