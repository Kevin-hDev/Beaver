use super::contract::{ContinuationUse, ReasoningModeId};
use super::registry::{ModelPolicy, ReplayRequirement};
use super::registry_inventory::{disabled, live};

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
    let fixture_id = match reasoning_mode {
        ReasoningModeId::Low => "anthropic-api-claude-haiku-4-5-20251001-low-france-2026-08-29",
        ReasoningModeId::Medium => {
            "anthropic-api-claude-haiku-4-5-20251001-medium-france-2026-08-29"
        }
        ReasoningModeId::High => "anthropic-api-claude-haiku-4-5-20251001-high-france-2026-08-29",
        _ => return disabled(MODEL, reasoning_mode, continuation_use, requirement),
    };
    live(
        MODEL,
        reasoning_mode,
        continuation_use,
        requirement,
        fixture_id,
        "2026-08-29",
    )
}
