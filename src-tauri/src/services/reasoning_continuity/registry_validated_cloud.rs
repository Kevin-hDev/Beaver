use super::contract::{ContinuationUse, ReasoningModeId};
use super::registry::{ModelPolicy, ReplayRequirement};
use super::registry_inventory::live;

pub(super) const MISTRAL: &[ModelPolicy] = &[
    live(
        "mistral-small-2603",
        ReasoningModeId::High,
        ContinuationUse::UserContinuation,
        ReplayRequirement::Required,
        "mistral-api-mistral-small-2603-france-2026-08-26",
        "2026-08-26",
    ),
    live(
        "mistral-small-2603",
        ReasoningModeId::High,
        ContinuationUse::ToolContinuation,
        ReplayRequirement::Required,
        "mistral-api-mistral-small-2603-france-2026-08-26",
        "2026-08-26",
    ),
];

pub(super) const XAI_OAUTH: &[ModelPolicy] = &[
    live(
        "grok-4.6",
        ReasoningModeId::High,
        ContinuationUse::UserContinuation,
        ReplayRequirement::Required,
        "xai-oauth-grok-4-6-local-2026-08-26",
        "2026-08-26",
    ),
    live(
        "grok-4.6",
        ReasoningModeId::High,
        ContinuationUse::ToolContinuation,
        ReplayRequirement::Required,
        "xai-oauth-grok-4-6-local-2026-08-26",
        "2026-08-26",
    ),
];
