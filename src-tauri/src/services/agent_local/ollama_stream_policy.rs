use super::types_ollama::{ChatRequest, OllamaThink};
use crate::services::compress::realtime_budget::RealtimeBudget;
use crate::services::reasoning_continuity::contract::ReasoningModeId;

pub(super) fn reasoning_mode(request: &ChatRequest) -> ReasoningModeId {
    match request.think.as_ref() {
        Some(OllamaThink::Level(level)) =>
            ReasoningModeId::from_name(Some(level)).unwrap_or(ReasoningModeId::Auto),
        Some(OllamaThink::Bool(true)) => ReasoningModeId::Auto,
        Some(OllamaThink::Bool(false)) | None => ReasoningModeId::Off,
    }
}

pub(super) fn should_interrupt(
    budget: &mut Option<RealtimeBudget>,
    token_count: u32,
    has_tool_call: bool,
) -> bool {
    !has_tool_call
        && budget
            .as_mut()
            .is_some_and(|budget| budget.should_interrupt(token_count))
}
