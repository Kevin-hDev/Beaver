use super::plan_mode_debug;
use super::types_ollama::{ChatMessage, StreamResult};
use super::types_plan::AgentPlanWorkflowStatus;

const MAX_REPAIRS: usize = 4;

pub enum PlanModeDecision {
    Accept,
    Retry(&'static str),
    Fail(&'static str),
}

pub async fn evaluate(
    session_id: &str,
    result: &StreamResult,
    repair_count: usize,
) -> PlanModeDecision {
    let Ok(session) = super::session_store::get(session_id).await else {
        return PlanModeDecision::Accept;
    };
    if !session.plan_mode_enabled {
        return PlanModeDecision::Accept;
    }
    let decision = decide(session.plan_workflow_status, result, repair_count);
    plan_mode_debug::controller_decision(
        session_id,
        session.plan_workflow_status,
        repair_count,
        result,
        &decision,
    );
    decision
}

pub(crate) fn decide(
    workflow: AgentPlanWorkflowStatus,
    result: &StreamResult,
    repair_count: usize,
) -> PlanModeDecision {
    match workflow {
        AgentPlanWorkflowStatus::NeedsContext | AgentPlanWorkflowStatus::CollectingQuestions => {
            decide_before_plan(result, repair_count)
        }
        AgentPlanWorkflowStatus::PlanPublished
        | AgentPlanWorkflowStatus::AwaitingApproval
        | AgentPlanWorkflowStatus::Approved
        | AgentPlanWorkflowStatus::Rejected => PlanModeDecision::Accept,
        AgentPlanWorkflowStatus::Cancelled => PlanModeDecision::Fail("Plan Mode was cancelled."),
    }
}

pub fn correction_message(content: &'static str) -> ChatMessage {
    ChatMessage {
        role: "system".to_string(),
        content: content.to_string(),
        ..Default::default()
    }
}

fn decide_before_plan(result: &StreamResult, repair_count: usize) -> PlanModeDecision {
    if !result.tool_calls.is_empty() || asks_user_question(&result.content) {
        return PlanModeDecision::Accept;
    }
    repair_or_fail(repair_count, NEXT_ACTION_REPAIR)
}

fn repair_or_fail(repair_count: usize, correction: &'static str) -> PlanModeDecision {
    if repair_count >= MAX_REPAIRS {
        PlanModeDecision::Fail("Plan Mode workflow could not be enforced.")
    } else {
        PlanModeDecision::Retry(correction)
    }
}

fn asks_user_question(content: &str) -> bool {
    content.contains('?') || content.contains('？')
}

const NEXT_ACTION_REPAIR: &str = "\
<plan_mode_backend_correction>
Plan Mode is active. You can ask important clarification questions in normal assistant text, call planmode if the plan is ready, or use read-only tools if more context is needed.
</plan_mode_backend_correction>";

#[cfg(test)]
#[path = "plan_mode_controller_tests.rs"]
mod tests;
