use super::interactive_choice_gate::InteractiveChoiceResponse;
use super::stream_events::AgentEventEmitter;
use super::tool_plan_approval_request::APPROVAL_ID_IMPLEMENT;
use super::types_interactive::AgentInteractiveAnswer;
use super::types_plan::{AgentPlanStatus, AgentPlanWorkflowStatus};
use super::types_session::AgentSession;
use super::types_stream::StreamEvent;

const ADJUSTMENTS_ID: &str = "other";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanApprovalOutcome {
    Implement,
    Adjustments(String),
    Dismissed,
}

pub async fn apply_response(
    session_id: &str,
    response: InteractiveChoiceResponse,
    on_event: &AgentEventEmitter,
) -> Result<PlanApprovalOutcome, String> {
    let outcome = classify_response(response)?;
    let lock = super::session_store::lock_session(session_id).await;
    let _guard = lock.lock().await;
    let mut session = super::session_store::get(session_id).await?;
    if !session.plan_mode_enabled
        || session.plan_workflow_status != AgentPlanWorkflowStatus::AwaitingApproval
    {
        return Err("Plan approval is no longer available.".to_string());
    }

    apply_outcome(&mut session, &outcome);
    super::session_store::save(&session).await?;
    emit_state(on_event, &outcome);
    Ok(outcome)
}

pub(crate) fn classify_response(
    response: InteractiveChoiceResponse,
) -> Result<PlanApprovalOutcome, String> {
    match response {
        InteractiveChoiceResponse::Dismissed => Ok(PlanApprovalOutcome::Dismissed),
        InteractiveChoiceResponse::Answered(answers) => classify_answers(&answers),
    }
}

fn classify_answers(answers: &[AgentInteractiveAnswer]) -> Result<PlanApprovalOutcome, String> {
    let [answer] = answers else {
        return Err("Invalid plan approval response.".to_string());
    };
    match answer.selected_ids.as_slice() {
        [id] if id == APPROVAL_ID_IMPLEMENT && answer.custom_answer.is_none() => {
            Ok(PlanApprovalOutcome::Implement)
        }
        [id] if id == ADJUSTMENTS_ID => answer
            .custom_answer
            .as_deref()
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .map(|text| PlanApprovalOutcome::Adjustments(text.to_string()))
            .ok_or_else(|| "Plan adjustments are required.".to_string()),
        _ => Err("Invalid plan approval response.".to_string()),
    }
}

fn apply_outcome(session: &mut AgentSession, outcome: &PlanApprovalOutcome) {
    let (run_status, workflow, enabled) = match outcome {
        PlanApprovalOutcome::Implement => (
            AgentPlanStatus::Approved,
            AgentPlanWorkflowStatus::Approved,
            false,
        ),
        PlanApprovalOutcome::Adjustments(_) => (
            AgentPlanStatus::Rejected,
            AgentPlanWorkflowStatus::CollectingQuestions,
            true,
        ),
        PlanApprovalOutcome::Dismissed => (
            AgentPlanStatus::Cancelled,
            AgentPlanWorkflowStatus::CollectingQuestions,
            true,
        ),
    };
    mark_active_run(session, run_status);
    session.plan_mode_enabled = enabled;
    session.plan_workflow_status = workflow;
    session.active_plan_id = None;
}

fn mark_active_run(session: &mut AgentSession, status: AgentPlanStatus) {
    if let Some(active_id) = session.active_plan_id.as_deref() {
        if let Some(run) = session.plan_runs.iter_mut().find(|run| run.id == active_id) {
            run.status = status;
            run.updated_at = chrono::Utc::now();
        }
    }
}

fn emit_state(on_event: &AgentEventEmitter, outcome: &PlanApprovalOutcome) {
    let _ = on_event.send(StreamEvent::PlanPreviewUpdated { plan: None });
    if outcome == &PlanApprovalOutcome::Implement {
        let _ = on_event.send(StreamEvent::PlanModeUpdated { enabled: false });
    }
}

#[cfg(test)]
#[path = "tool_plan_approval_tests.rs"]
mod tests;
