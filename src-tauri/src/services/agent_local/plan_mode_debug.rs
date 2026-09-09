use super::plan_mode_controller::PlanModeDecision;
use super::types_ollama::StreamResult;
use super::types_plan::AgentPlanWorkflowStatus;

pub fn controller_decision(
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
    ::log::info!(
        "[plan-mode] decision workflow={workflow:?} repairs={repair_count} content_chars={} question={} tool_calls={} decision={decision_label}",
        result.content.chars().count(),
        has_question(&result.content),
        result.tool_calls.len(),
    );
}

pub fn workflow_failed(message: &str) {
    ::log::error!(
        "[plan-mode] failed reason={}",
        failure_code(message),
    );
}

fn has_question(content: &str) -> bool {
    content.contains('?') || content.contains('？')
}

fn failure_code(message: &str) -> &'static str {
    match message {
        "Plan Mode was cancelled." => "cancelled",
        "Plan Mode workflow could not be enforced." => "enforcement_failed",
        _ => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::failure_code;

    #[test]
    fn failure_codes_never_echo_untrusted_text() {
        assert_eq!(failure_code("Plan Mode was cancelled."), "cancelled");
        assert_eq!(
            failure_code("Plan Mode workflow could not be enforced."),
            "enforcement_failed"
        );
        assert_eq!(failure_code("token=secret-value"), "unknown");
    }
}
