use super::*;

#[test]
fn detects_plain_text_questions() {
    assert!(asks_user_question("Which target should I use?"));
    assert!(!asks_user_question("I will inspect the project."));
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
fn published_and_terminal_states_never_request_a_hidden_follow_up() {
    let result = StreamResult::default();
    for workflow in [
        AgentPlanWorkflowStatus::PlanPublished,
        AgentPlanWorkflowStatus::AwaitingApproval,
        AgentPlanWorkflowStatus::Approved,
        AgentPlanWorkflowStatus::Rejected,
    ] {
        assert!(matches!(
            decide(workflow, &result, 0),
            PlanModeDecision::Accept
        ));
    }
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
