use super::contract::{ContinuationUse, ReasoningModeId};
use super::registry::{ModelPolicy, ReplayRequirement};
use super::registry_inventory::{disabled, live};

const MODEL: &str = "qwen3.8-flash";
const UNVALIDATED_MODEL: &str = "qwen3.8-max";

pub(super) const QWEN: &[ModelPolicy] = &[
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
        ReasoningModeId::Xhigh,
        ContinuationUse::UserContinuation,
        ReplayRequirement::Required,
    ),
    policy(
        ReasoningModeId::Xhigh,
        ContinuationUse::ToolContinuation,
        ReplayRequirement::Required,
    ),
    disabled(
        UNVALIDATED_MODEL,
        ReasoningModeId::Low,
        ContinuationUse::UserContinuation,
        ReplayRequirement::Required,
    ),
    disabled(
        UNVALIDATED_MODEL,
        ReasoningModeId::Low,
        ContinuationUse::ToolContinuation,
        ReplayRequirement::Required,
    ),
    disabled(
        UNVALIDATED_MODEL,
        ReasoningModeId::Medium,
        ContinuationUse::UserContinuation,
        ReplayRequirement::Required,
    ),
    disabled(
        UNVALIDATED_MODEL,
        ReasoningModeId::Medium,
        ContinuationUse::ToolContinuation,
        ReplayRequirement::Required,
    ),
    disabled(
        UNVALIDATED_MODEL,
        ReasoningModeId::Xhigh,
        ContinuationUse::UserContinuation,
        ReplayRequirement::Required,
    ),
    disabled(
        UNVALIDATED_MODEL,
        ReasoningModeId::Xhigh,
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
        ReasoningModeId::Low => "qwen-api-qwen3-8-flash-low-singapore-2026-08-29",
        ReasoningModeId::Medium => "qwen-api-qwen3-8-flash-medium-singapore-2026-08-29",
        ReasoningModeId::Xhigh => "qwen-api-qwen3-8-flash-xhigh-singapore-2026-08-29",
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
