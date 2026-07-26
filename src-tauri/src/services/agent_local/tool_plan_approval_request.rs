use tokio_util::sync::CancellationToken;

use super::stream_events::AgentEventEmitter;
use super::types_interactive::{AgentInteractiveOption, AgentInteractiveQuestion};
use super::types_tools::ToolResult;

pub const APPROVAL_ID_IMPLEMENT: &str = "implement_plan";
pub const APPROVAL_ID_CONTINUE: &str = "continue_planning";
pub const APPROVAL_ID_QUIT: &str = "quit_plan";

pub async fn request_approval(
    on_event: &AgentEventEmitter,
    session_id: &str,
    cancel: CancellationToken,
    title: &str,
) -> ToolResult {
    let questions = vec![approval_question()];
    match super::interactive_choice_gate::request(on_event, session_id, questions, cancel).await {
        Ok(answers) => {
            match super::tool_plan_approval::record_answers(session_id, &answers, on_event).await {
                Ok(decision) => {
                    ToolResult::ok(super::tool_plan_messages::published(title, decision))
                }
                Err(err) => ToolResult::err(err),
            }
        }
        Err(err) => ToolResult::err(err),
    }
}

pub(crate) fn approval_question() -> AgentInteractiveQuestion {
    AgentInteractiveQuestion {
        header: "Plan".to_string(),
        question: "Mettre en oeuvre le plan ?".to_string(),
        multi_select: false,
        options: vec![
            AgentInteractiveOption {
                id: Some(APPROVAL_ID_IMPLEMENT.to_string()),
                label: "Mettre en oeuvre le plan".to_string(),
                description: "Valider le plan et lancer l'implementation.".to_string(),
                recommended: true,
                preview: None,
            },
            AgentInteractiveOption {
                id: Some(APPROVAL_ID_CONTINUE.to_string()),
                label: "Continuer a planifier".to_string(),
                description: "Reprendre le mode plan et ajuster le plan.".to_string(),
                recommended: false,
                preview: None,
            },
            AgentInteractiveOption {
                id: Some(APPROVAL_ID_QUIT.to_string()),
                label: "Quitter le mode plan".to_string(),
                description: "Refuser ce plan et sortir du mode plan.".to_string(),
                recommended: false,
                preview: None,
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn approval_question_uses_expected_choices() {
        let question = super::approval_question();
        assert_eq!(question.question, "Mettre en oeuvre le plan ?");
        assert_eq!(question.options.len(), 3);
        assert!(question.options[0].recommended);
        assert_eq!(question.options[0].label, "Mettre en oeuvre le plan");
        assert_eq!(question.options[1].label, "Continuer a planifier");
        assert_eq!(question.options[2].label, "Quitter le mode plan");
        assert_eq!(
            question.options[0].id.as_deref(),
            Some(super::APPROVAL_ID_IMPLEMENT)
        );
    }
}
