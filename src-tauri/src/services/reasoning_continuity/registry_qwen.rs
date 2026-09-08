use super::contract::{ContinuationUse, ReasoningModeId};
use super::registry::{ModelPolicy, ReplayRequirement};
use super::registry_inventory::{disabled, live};

const MODEL: &str = "qwen3.8-flash";

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
    // Alibaba's exact third-party contract still lacks a safe output-limit field;
    // block it instead of inheriting Qwen transport or inventing a payload.
    disabled(
        "ZHIPU/GLM-5.3-Flash",
        ReasoningModeId::Low,
        ContinuationUse::UserContinuation,
        ReplayRequirement::Forbidden,
    ),
    disabled(
        "ZHIPU/GLM-5.3-Flash",
        ReasoningModeId::Low,
        ContinuationUse::ToolContinuation,
        ReplayRequirement::Forbidden,
    ),
    disabled(
        "ZHIPU/GLM-5.3-Flash",
        ReasoningModeId::High,
        ContinuationUse::UserContinuation,
        ReplayRequirement::Forbidden,
    ),
    disabled(
        "ZHIPU/GLM-5.3-Flash",
        ReasoningModeId::High,
        ContinuationUse::ToolContinuation,
        ReplayRequirement::Forbidden,
    ),
    disabled(
        "ZHIPU/GLM-5.3-Flash",
        ReasoningModeId::Max,
        ContinuationUse::UserContinuation,
        ReplayRequirement::Forbidden,
    ),
    disabled(
        "ZHIPU/GLM-5.3-Flash",
        ReasoningModeId::Max,
        ContinuationUse::ToolContinuation,
        ReplayRequirement::Forbidden,
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
