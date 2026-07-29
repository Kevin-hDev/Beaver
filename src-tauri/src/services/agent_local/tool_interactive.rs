use tokio_util::sync::CancellationToken;

use super::interactive_choice_gate::InteractiveChoiceResponse;
use super::stream_events::AgentEventEmitter;
use super::types_interactive::{
    AgentInteractiveAnswer, AgentInteractiveChoiceKind, AgentInteractiveQuestion,
};
use super::types_ollama::StreamEvent;
use super::types_tools::ToolResult;

pub async fn execute(
    args: &serde_json::Value,
    on_event: &AgentEventEmitter,
    cancel: CancellationToken,
    session_id: Option<&str>,
) -> ToolResult {
    let questions = match super::tool_interactive_parse::parse_questions(args) {
        Ok(questions) => questions,
        Err(err) => return ToolResult::err(err),
    };
    let Some(session_id) = session_id else {
        return ToolResult::err("Contexte interactif indisponible.");
    };
    match request(on_event, session_id, questions.clone(), cancel).await {
        Ok(InteractiveChoiceResponse::Answered(answers)) => answered_result(&questions, &answers),
        Ok(InteractiveChoiceResponse::Dismissed) => {
            ToolResult::ok("Interactive choice dismissed.").stopping()
        }
        Err(err) => ToolResult::err(err),
    }
}

pub async fn respond(
    session_id: String,
    id: String,
    answers: Vec<AgentInteractiveAnswer>,
) -> Result<(), String> {
    super::interactive_choice_gate::respond(&session_id, &id, answers).await
}

pub async fn dismiss(session_id: String, id: String) -> Result<(), String> {
    super::interactive_choice_gate::dismiss(&session_id, &id).await
}

async fn request(
    on_event: &AgentEventEmitter,
    session_id: &str,
    questions: Vec<AgentInteractiveQuestion>,
    cancel: CancellationToken,
) -> Result<InteractiveChoiceResponse, String> {
    super::interactive_choice_gate::request(
        on_event,
        session_id,
        AgentInteractiveChoiceKind::General,
        questions,
        cancel,
    )
    .await
}

pub(crate) fn answered_result(
    questions: &[AgentInteractiveQuestion],
    answers: &[AgentInteractiveAnswer],
) -> ToolResult {
    ToolResult::ok("Interactive answer received.").with_user_message(user_reply(questions, answers))
}

fn user_reply(
    questions: &[AgentInteractiveQuestion],
    answers: &[AgentInteractiveAnswer],
) -> String {
    let mut lines = vec!["Interactive answers from the user:".to_string()];
    for answer in answers {
        if questions.get(answer.question_index).is_none() {
            continue;
        }
        let value = answer_value(answer);
        lines.push(format!("- Question {}: {value}", answer.question_index + 1));
    }
    lines.join("\n")
}

fn answer_value(answer: &AgentInteractiveAnswer) -> String {
    let mut values: Vec<_> = answer
        .selected_labels
        .iter()
        .filter(|label| label.as_str() != "other")
        .cloned()
        .collect();
    if values.is_empty() {
        values.extend(
            answer
                .selected_ids
                .iter()
                .filter(|id| id.as_str() != "other")
                .cloned(),
        );
    }
    if let Some(custom) = answer.custom_answer.as_deref() {
        values.push(custom.to_string());
    }
    values.join(", ")
}

pub(crate) fn emit_request(
    on_event: &AgentEventEmitter,
    session_id: String,
    id: String,
    kind: AgentInteractiveChoiceKind,
    questions: Vec<AgentInteractiveQuestion>,
) {
    let total = questions.len();
    let _ = on_event.send(StreamEvent::InteractiveChoiceRequest {
        session_id,
        id,
        kind,
        questions,
        current_index: 0,
        total,
    });
}
