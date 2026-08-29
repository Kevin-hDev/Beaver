use super::contract::{ContinuationUse, ReasoningModeId};
use super::registry::{ModelPolicy, ReplayRequirement};
use super::registry_inventory::disabled;

const MODEL: &str = "claude-haiku-4-5-20251001";

pub(super) const ANTHROPIC: &[ModelPolicy] = &[
    policy(
        ReasoningModeId::Off,
        ContinuationUse::UserContinuation,
        ReplayRequirement::Forbidden,
    ),
    policy(
        ReasoningModeId::Off,
        ContinuationUse::ToolContinuation,
        ReplayRequirement::Forbidden,
    ),
    policy(
        ReasoningModeId::Low,
        ContinuationUse::UserContinuation,
        ReplayRequirement::Required,
    ),
    policy(
        ReasoningModeId::Low,
        ContinuationUse::ToolContinuation,
        ReplayRequirement::Required,
    ),
    policy(
        ReasoningModeId::Medium,
        ContinuationUse::UserContinuation,
        ReplayRequirement::Required,
    ),
    policy(
        ReasoningModeId::Medium,
        ContinuationUse::ToolContinuation,
        ReplayRequirement::Required,
    ),
    policy(
        ReasoningModeId::High,
        ContinuationUse::UserContinuation,
        ReplayRequirement::Required,
    ),
    policy(
        ReasoningModeId::High,
        ContinuationUse::ToolContinuation,
        ReplayRequirement::Required,
    ),
];

const fn policy(
    reasoning_mode: ReasoningModeId,
    continuation_use: ContinuationUse,
    requirement: ReplayRequirement,
) -> ModelPolicy {
    disabled(MODEL, reasoning_mode, continuation_use, requirement)
}
