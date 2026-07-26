use serde_json::json;
use tokio_util::sync::CancellationToken;

use super::stream_events::AgentEventEmitter;
use super::types_interactive::{AgentInteractiveAnswer, AgentInteractiveQuestion};
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
    match request(on_event, session_id, questions, cancel).await {
        Ok(answers) => {
            let plan_decision =
                super::tool_plan_approval::record_answers(session_id, &answers, on_event)
                    .await
                    .ok()
                    .flatten();
            ToolResult::ok(result_json(&answers, plan_decision))
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

async fn request(
    on_event: &AgentEventEmitter,
    session_id: &str,
    questions: Vec<AgentInteractiveQuestion>,
    cancel: CancellationToken,
) -> Result<Vec<AgentInteractiveAnswer>, String> {
    super::interactive_choice_gate::request(on_event, session_id, questions, cancel).await
}

fn result_json(answers: &[AgentInteractiveAnswer], plan_decision: Option<&str>) -> String {
    let selected_labels: Vec<_> = answers
        .iter()
        .map(|answer| answer.selected_labels.clone())
        .collect();
    let selected_ids: Vec<_> = answers
        .iter()
        .map(|answer| answer.selected_ids.clone())
        .collect();
    let custom_answers: Vec<_> = answers
        .iter()
        .filter_map(|answer| answer.custom_answer.clone())
        .collect();
    let mut value = json!({
        "completed": true,
        "answers": answers,
        "selected_ids": selected_ids,
        "selected_labels": selected_labels,
        "custom_answers": custom_answers,
    });
    if let Some(decision) = plan_decision {
        value["plan_mode_decision"] = json!(decision);
    }
    value.to_string()
}

pub(crate) fn emit_request(
    on_event: &AgentEventEmitter,
    session_id: String,
    id: String,
    questions: Vec<AgentInteractiveQuestion>,
) {
    let total = questions.len();
    let _ = on_event.send(StreamEvent::InteractiveChoiceRequest {
        session_id,
        id,
        questions,
        current_index: 0,
        total,
    });
}
