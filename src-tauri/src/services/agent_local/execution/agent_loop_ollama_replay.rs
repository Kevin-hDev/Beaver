use super::types_ollama::ChatMessage;
use crate::services::reasoning_continuity::contract::{ContinuationUse, ReplayTarget};
use crate::services::reasoning_continuity::registry::{ActivationState, ReplayRequirement};

pub(super) fn follows_tool_result(messages: &[ChatMessage]) -> bool {
    messages
        .last()
        .is_some_and(|message| message.role == "tool")
}

pub(super) fn for_request(
    target: &ReplayTarget,
    follows_tool_result: bool,
) -> Result<ReplayTarget, String> {
    let mut target = target.clone();
    target.continuation_use = if follows_tool_result {
        ContinuationUse::ToolContinuation
    } else {
        ContinuationUse::UserContinuation
    };
    let allowed = crate::services::reasoning_continuity::registry::replay_policy(&target)
        .is_some_and(|policy| {
            policy.activation() == ActivationState::LiveValidated
                && policy.requirement() != ReplayRequirement::Forbidden
        });
    allowed
        .then_some(target)
        .ok_or_else(|| "reasoning_continuity_invalid".to_string())
}
