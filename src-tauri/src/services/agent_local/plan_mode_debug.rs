use super::plan_mode_controller::PlanModeDecision;
use super::types_plan::AgentPlanWorkflowStatus;

pub fn controller_decision(
    workflow: AgentPlanWorkflowStatus,
    repair_count: usize,
    decision: &PlanModeDecision,
) {
    let decision_label = match decision {
        PlanModeDecision::Accept => "accept",
        PlanModeDecision::Retry(_) => "retry",
        PlanModeDecision::Fail(_) => "fail",
    };
    ::log::info!(
        "[plan-mode] decision workflow={workflow:?} repairs={repair_count} decision={decision_label}",
    );
}

pub fn workflow_failed(message: &str) {
    ::log::error!("[plan-mode] failed reason={}", failure_code(message),);
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
