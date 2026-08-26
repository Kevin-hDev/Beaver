use super::contract::{ContinuationUse, ReasoningModeId};
use super::registry::{ModelPolicy, ReplayRequirement};
use super::registry_inventory::{disabled, live};

// Ces couples sont activés uniquement après un rapport live passe/passe daté.
// Les modèles voisins restent fermés : un format commun ne prouve pas leur compatibilité.
pub(super) const GOOGLE: &[ModelPolicy] = &[
    disabled(
        "gemini-3.7-flash",
        ReasoningModeId::Medium,
        ContinuationUse::UserContinuation,
        ReplayRequirement::Required,
    ),
    disabled(
        "gemini-3.7-flash",
        ReasoningModeId::Medium,
        ContinuationUse::ToolContinuation,
        ReplayRequirement::Required,
    ),
    live(
        "gemini-3.5-flash",
        ReasoningModeId::Medium,
        ContinuationUse::UserContinuation,
        ReplayRequirement::Required,
        "google-api-gemini-3-5-flash-france-2026-08-26",
        "2026-08-26",
    ),
    live(
        "gemini-3.5-flash",
        ReasoningModeId::Medium,
        ContinuationUse::ToolContinuation,
        ReplayRequirement::Required,
        "google-api-gemini-3-5-flash-france-2026-08-26",
        "2026-08-26",
    ),
    disabled(
        "gemini-3.5-flash-lite",
        ReasoningModeId::Medium,
        ContinuationUse::UserContinuation,
        ReplayRequirement::Required,
    ),
    disabled(
        "gemini-3.5-flash-lite",
        ReasoningModeId::Medium,
        ContinuationUse::ToolContinuation,
        ReplayRequirement::Required,
    ),
];

pub(super) const CEREBRAS: &[ModelPolicy] = &[
    disabled(
        "zai-glm-4.7",
        ReasoningModeId::Auto,
        ContinuationUse::UserContinuation,
        ReplayRequirement::Optional,
    ),
    disabled(
        "zai-glm-4.7",
        ReasoningModeId::Auto,
        ContinuationUse::ToolContinuation,
        ReplayRequirement::Optional,
    ),
    live(
        "gpt-oss-120b",
        ReasoningModeId::High,
        ContinuationUse::UserContinuation,
        ReplayRequirement::Required,
        "cerebras-api-gpt-oss-120b-france-2026-08-26",
        "2026-08-26",
    ),
    live(
        "gpt-oss-120b",
        ReasoningModeId::High,
        ContinuationUse::ToolContinuation,
        ReplayRequirement::Required,
        "cerebras-api-gpt-oss-120b-france-2026-08-26",
        "2026-08-26",
    ),
];

pub(super) const OPENROUTER: &[ModelPolicy] = &[
    live(
        "moonshotai/kimi-k2.5",
        ReasoningModeId::Medium,
        ContinuationUse::UserContinuation,
        ReplayRequirement::Required,
        "openrouter-api-moonshotai-kimi-k2-5-france-2026-08-26",
        "2026-08-26",
    ),
    live(
        "moonshotai/kimi-k2.5",
        ReasoningModeId::Medium,
        ContinuationUse::ToolContinuation,
        ReplayRequirement::Required,
        "openrouter-api-moonshotai-kimi-k2-5-france-2026-08-26",
        "2026-08-26",
    ),
];

pub(super) const DEEPSEEK: &[ModelPolicy] = &[
    disabled(
        "deepseek-v4-flash",
        ReasoningModeId::High,
        ContinuationUse::UserContinuation,
        ReplayRequirement::Forbidden,
    ),
    live(
        "deepseek-v4-flash",
        ReasoningModeId::High,
        ContinuationUse::ToolContinuation,
        ReplayRequirement::Required,
        "deepseek-api-deepseek-v4-flash-france-2026-08-26",
        "2026-08-26",
    ),
];

pub(super) const XAI: &[ModelPolicy] = &[
    live(
        "grok-4.6",
        ReasoningModeId::High,
        ContinuationUse::UserContinuation,
        ReplayRequirement::Required,
        "xai-api-grok-4-6-france-2026-08-26",
        "2026-08-26",
    ),
    live(
        "grok-4.6",
        ReasoningModeId::High,
        ContinuationUse::ToolContinuation,
        ReplayRequirement::Required,
        "xai-api-grok-4-6-france-2026-08-26",
        "2026-08-26",
    ),
];

pub(super) const MOONSHOT: &[ModelPolicy] = &[
    live(
        "kimi-k2.7-code",
        ReasoningModeId::Auto,
        ContinuationUse::UserContinuation,
        ReplayRequirement::Required,
        "moonshot-api-kimi-k2-7-code-france-2026-08-26",
        "2026-08-26",
    ),
    live(
        "kimi-k2.7-code",
        ReasoningModeId::Auto,
        ContinuationUse::ToolContinuation,
        ReplayRequirement::Required,
        "moonshot-api-kimi-k2-7-code-france-2026-08-26",
        "2026-08-26",
    ),
];
