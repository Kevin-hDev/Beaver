use super::types_plan::AgentPlanStatus;

pub const INVALID_PLAN: &str = "Invalid plan.";
pub const INVALID_STATUS: &str = "Invalid plan status.";
pub const PLAN_INACTIVE: &str = "Plan Mode is not active.";
pub const PLAN_UNAVAILABLE: &str = "Plan unavailable.";

pub fn published(title: &str, decision: Option<&str>) -> String {
    match decision {
        Some("implement") => format!(
            "Plan '{title}' has been published and approved by the user. \
             Call exitplanmode with status approved now. After it succeeds, immediately start implementation."
        ),
        Some("continue_planning") => format!(
            "Plan '{title}' has been published. The user chose to continue planning. \
             Stay in Plan Mode, adjust the plan, then call planmode again when ready."
        ),
        Some("quit_plan") => format!(
            "Plan '{title}' has been published. The user chose to quit Plan Mode. \
             Call exitplanmode with status rejected now."
        ),
        _ => format!(
            "Plan '{title}' has been published, but the approval choice was not recognized. \
             Stay in Plan Mode and ask for clarification."
        ),
    }
}

pub fn exited(status: AgentPlanStatus) -> &'static str {
    match status {
        AgentPlanStatus::Approved => {
            "Plan Mode exited. The plan is approved. todo_write and write tools are available again. \
             You must immediately start implementation now without waiting for another user message."
        }
        AgentPlanStatus::Rejected => "Plan Mode exited. The plan is not approved.",
        AgentPlanStatus::Cancelled => "Plan Mode cancelled.",
        _ => "Plan Mode exited.",
    }
}
