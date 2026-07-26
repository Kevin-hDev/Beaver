use super::plan_mode_controller::{self, PlanModeDecision};
use super::stream_events::AgentEventEmitter;
use super::types_ollama::{ChatMessage, StreamResult};

pub enum PlanLoopAction {
    Accept,
    Retry,
    Stop(&'static str),
}

pub async fn active(session_id: &str, fallback: bool) -> bool {
    fallback && super::tool_plan::is_enabled(session_id).await
}

pub async fn check_result(
    _on_event: &AgentEventEmitter,
    messages: &mut Vec<ChatMessage>,
    session_id: &str,
    request_id: &str,
    result: &StreamResult,
    active: bool,
    repair_count: usize,
) -> PlanLoopAction {
    if !active {
        return PlanLoopAction::Accept;
    }
    match plan_mode_controller::evaluate(session_id, result, repair_count).await {
        PlanModeDecision::Accept => PlanLoopAction::Accept,
        PlanModeDecision::Retry(correction) => {
            replace_correction(
                messages,
                plan_mode_controller::correction_message(correction),
            );
            PlanLoopAction::Retry
        }
        PlanModeDecision::Fail(message) => {
            super::plan_mode_debug::workflow_failed(session_id, request_id, message);
            PlanLoopAction::Stop(message)
        }
    }
}

fn replace_correction(messages: &mut Vec<ChatMessage>, correction: ChatMessage) {
    messages.retain(|message| !is_plan_correction(message));
    messages.push(correction);
}

fn is_plan_correction(message: &ChatMessage) -> bool {
    message.role == "system" && message.content.contains("<plan_mode_backend_correction>")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replaces_existing_plan_correction() {
        let mut messages = vec![
            ChatMessage {
                role: "system".into(),
                content: "<plan_mode_backend_correction>old</plan_mode_backend_correction>".into(),
                ..Default::default()
            },
            ChatMessage {
                role: "user".into(),
                content: "hello".into(),
                ..Default::default()
            },
        ];
        replace_correction(
            &mut messages,
            ChatMessage {
                role: "system".into(),
                content: "<plan_mode_backend_correction>new</plan_mode_backend_correction>".into(),
                ..Default::default()
            },
        );

        assert_eq!(messages.len(), 2);
        assert!(messages[1].content.contains("new"));
    }
}
