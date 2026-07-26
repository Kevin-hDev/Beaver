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
        AgentPlanWorkflowStatus::PlanPublished | AgentPlanWorkflowStatus::AwaitingApproval => {
            repair_or_fail(repair_count, APPROVAL_REPAIR)
        }
        AgentPlanWorkflowStatus::Approved => {
            if has_tool(result, "exitplanmode") {
                PlanModeDecision::Accept
            } else {
                repair_or_fail(repair_count, EXIT_REPAIR)
            }
        }
        AgentPlanWorkflowStatus::Rejected => {
            if has_tool(result, "exitplanmode") {
                PlanModeDecision::Accept
            } else {
                repair_or_fail(repair_count, REJECTED_REPAIR)
            }
        }
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

fn has_tool(result: &StreamResult, name: &str) -> bool {
    result.tool_calls.iter().any(|(tool, _)| tool == name)
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

const APPROVAL_REPAIR: &str = "\
<plan_mode_backend_correction>
The plan is published and waiting for final approval. Do not ask approval yourself. Call planmode again only if you need to update the plan; otherwise wait for the backend approval result.
</plan_mode_backend_correction>";

const EXIT_REPAIR: &str = "\
<plan_mode_backend_correction>
The user approved the plan. Call exitplanmode with status approved now. After it succeeds, immediately start implementation.
</plan_mode_backend_correction>";

const REJECTED_REPAIR: &str = "\
<plan_mode_backend_correction>
The user chose to quit Plan Mode. Call exitplanmode with status rejected now.
</plan_mode_backend_correction>";

const NEXT_ACTION_REPAIR: &str = "\
<plan_mode_backend_correction>
Plan Mode is active. You can ask important clarification questions in normal assistant text, call planmode if the plan is ready, or use read-only tools if more context is needed.
</plan_mode_backend_correction>";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_plain_text_questions() {
        assert!(asks_user_question("Which target should I use?"));
        assert!(!asks_user_question("I will inspect the project."));
    }

    #[test]
    fn finds_tool_calls_by_name() {
        let result = StreamResult {
            tool_calls: vec![("ask_user_choice".into(), serde_json::json!({}))],
            ..Default::default()
        };
        assert!(has_tool(&result, "ask_user_choice"));
        assert!(!has_tool(&result, "planmode"));
    }

    #[test]
    fn accepts_plain_text_questions_before_plan_publish() {
        let result = StreamResult {
            content: "Which option should I use?".into(),
            ..Default::default()
        };
        assert!(matches!(
            decide(AgentPlanWorkflowStatus::NeedsContext, &result, 0),
            PlanModeDecision::Accept
        ));
    }

    #[test]
    fn repairs_plain_text_questions_after_plan_publish() {
        let result = StreamResult {
            content: "Should I implement it?".into(),
            ..Default::default()
        };
        assert!(matches!(
            decide(AgentPlanWorkflowStatus::AwaitingApproval, &result, 0),
            PlanModeDecision::Retry(_)
        ));
    }

    #[test]
    fn plan_published_legacy_state_requires_approval_flow() {
        let result = StreamResult {
            tool_calls: vec![("ask_user_choice".into(), serde_json::json!({}))],
            ..Default::default()
        };
        assert!(matches!(
            decide(AgentPlanWorkflowStatus::PlanPublished, &result, 0),
            PlanModeDecision::Retry(_)
        ));
    }

    #[test]
    fn requires_final_approval_after_plan_is_published() {
        let result = StreamResult::default();
        assert!(matches!(
            decide(AgentPlanWorkflowStatus::AwaitingApproval, &result, 0),
            PlanModeDecision::Retry(_)
        ));
    }

    #[test]
    fn fails_after_too_many_repairs() {
        let result = StreamResult::default();
        assert!(matches!(
            decide(AgentPlanWorkflowStatus::NeedsContext, &result, MAX_REPAIRS),
            PlanModeDecision::Fail(_)
        ));
    }

    #[test]
    fn cancelled_state_fails_explicitly() {
        let result = StreamResult::default();
        assert!(matches!(
            decide(AgentPlanWorkflowStatus::Cancelled, &result, 0),
            PlanModeDecision::Fail(_)
        ));
    }

    #[test]
    fn still_repairs_before_limit() {
        let result = StreamResult::default();
        assert!(matches!(
            decide(
                AgentPlanWorkflowStatus::NeedsContext,
                &result,
                MAX_REPAIRS - 1
            ),
            PlanModeDecision::Retry(_)
        ));
    }

    #[test]
    fn accepts_valid_tool_even_after_repairs() {
        let result = StreamResult {
            tool_calls: vec![("ask_user_choice".into(), serde_json::json!({}))],
            ..Default::default()
        };
        assert!(matches!(
            decide(AgentPlanWorkflowStatus::NeedsContext, &result, MAX_REPAIRS),
            PlanModeDecision::Accept
        ));
    }
}
