#![allow(
    dead_code,
    reason = "the compression orchestrator consumes reasoning selection in Task 10"
)]

use crate::services::agent_local::types_session::AgentMessage;

pub fn validate(message: &AgentMessage) -> Result<(), &'static str> {
    let Some(envelope) = message.continuation.as_ref() else {
        return Ok(());
    };
    envelope
        .validate()
        .map_err(|_| "compression_checkpoint_invalid_reasoning")?;
    crate::services::reasoning_continuity::eligibility::state_matches_contract(
        envelope.contract_id,
        &envelope.continuation,
    )
    .then_some(())
    .ok_or("compression_checkpoint_invalid_reasoning")
}
