use tokio_util::sync::CancellationToken;

use super::stream_events::AgentEventEmitter;
use super::tool_plan_approval::PlanApprovalOutcome;
use super::types_interactive::{
    AgentInteractiveChoiceKind, AgentInteractiveOption, AgentInteractiveQuestion,
};
use super::types_tools::ToolResult;

pub const APPROVAL_ID_IMPLEMENT: &str = "implement_plan";

const IMPLEMENT_FOLLOW_UP: &str = "\
The user approved the plan. Plan Mode is already closed. \
Start implementing the approved plan now.";

pub async fn request_approval(
    on_event: &AgentEventEmitter,
    session_id: &str,
    cancel: CancellationToken,
) -> ToolResult {
    let response = super::interactive_choice_gate::request(
        on_event,
        session_id,
        AgentInteractiveChoiceKind::PlanApproval,
        vec![approval_question()],
        cancel,
    )
    .await;
    let response = match response {
        Ok(response) => response,
        Err(err) => return ToolResult::err(err),
    };

    match super::tool_plan_approval::apply_response(session_id, response, on_event).await {
        Ok(outcome) => result_for_outcome(outcome),
        Err(err) => ToolResult::err(err),
    }
}

pub(crate) fn result_for_outcome(outcome: PlanApprovalOutcome) -> ToolResult {
    match outcome {
        PlanApprovalOutcome::Implement => {
            ToolResult::ok("Plan approval recorded.").with_system_message(IMPLEMENT_FOLLOW_UP)
        }
        PlanApprovalOutcome::Adjustments(text) => {
            ToolResult::ok("Plan adjustments recorded.")
                .with_user_message(format!("Plan adjustments from the user:\n{text}"))
        }
        PlanApprovalOutcome::Dismissed => {
            ToolResult::ok("Plan approval dismissed.").stopping()
        }
    }
}

pub(crate) fn approval_question() -> AgentInteractiveQuestion {
    AgentInteractiveQuestion {
        header: "Plan".to_string(),
        question: "Mettre en oeuvre le plan ?".to_string(),
        multi_select: false,
        options: vec![AgentInteractiveOption {
            id: Some(APPROVAL_ID_IMPLEMENT.to_string()),
            label: "Mettre en oeuvre ce plan".to_string(),
            description: "Valider le plan et lancer l'implementation.".to_string(),
            recommended: true,
            preview: None,
        }],
    }
}

#[cfg(test)]
mod tests {
    use crate::services::agent_local::types_tools::ToolFollowUp;

    #[test]
    fn approval_question_exposes_only_implementation() {
        let question = super::approval_question();
        assert_eq!(question.options.len(), 1);
        assert!(question.options[0].recommended);
        assert_eq!(
            question.options[0].id.as_deref(),
            Some(super::APPROVAL_ID_IMPLEMENT)
        );
    }

    #[test]
    fn outcomes_use_their_real_message_authority() {
        let mut implement =
            super::result_for_outcome(super::PlanApprovalOutcome::Implement);
        assert!(matches!(
            implement.take_follow_up(),
            ToolFollowUp::SystemMessage(_)
        ));

        let mut adjustments = super::result_for_outcome(
            super::PlanApprovalOutcome::Adjustments("Change target".into()),
        );
        assert!(matches!(
            adjustments.take_follow_up(),
            ToolFollowUp::UserMessage(content) if content.contains("Change target")
        ));

        let mut dismissed =
            super::result_for_outcome(super::PlanApprovalOutcome::Dismissed);
        assert_eq!(dismissed.take_follow_up(), ToolFollowUp::Stop);
    }
}
