use super::contract::{ContinuationUse, ContractId, ReasoningModeId, RouteId};
use super::registry::{ActivationState, AdapterId, ModelPolicy, ReplayRequirement, RouteContract};
use super::registry_validated_cloud::{CODEX, OLLAMA, XAI_OAUTH, ZAI};

use ReplayRequirement::{Forbidden, Optional, Required};

const fn user(
    model_id: &'static str,
    reasoning_mode: ReasoningModeId,
    requirement: ReplayRequirement,
) -> ModelPolicy {
    disabled(
        model_id,
        reasoning_mode,
        ContinuationUse::UserContinuation,
        requirement,
    )
}

const fn tool(
    model_id: &'static str,
    reasoning_mode: ReasoningModeId,
    requirement: ReplayRequirement,
) -> ModelPolicy {
    disabled(
        model_id,
        reasoning_mode,
        ContinuationUse::ToolContinuation,
        requirement,
    )
}

pub(super) const fn disabled(
    model_id: &'static str,
    reasoning_mode: ReasoningModeId,
    continuation_use: ContinuationUse,
    requirement: ReplayRequirement,
) -> ModelPolicy {
    ModelPolicy {
        model_id,
        reasoning_mode,
        continuation_use,
        requirement,
        activation: ActivationState::Disabled,
        fixture_id: None,
        fixture_date: None,
    }
}

pub(super) const fn live(
    model_id: &'static str,
    reasoning_mode: ReasoningModeId,
    continuation_use: ContinuationUse,
    requirement: ReplayRequirement,
    fixture_id: &'static str,
    fixture_date: &'static str,
) -> ModelPolicy {
    ModelPolicy {
        model_id,
        reasoning_mode,
        continuation_use,
        requirement,
        activation: ActivationState::LiveValidated,
        fixture_id: Some(fixture_id),
        fixture_date: Some(fixture_date),
    }
}

const GOOGLE: &[ModelPolicy] = &[
    user("gemini-3.7-flash", ReasoningModeId::Medium, Required),
    tool("gemini-3.7-flash", ReasoningModeId::Medium, Required),
    user("gemini-3.5-flash", ReasoningModeId::Medium, Required),
    tool("gemini-3.5-flash", ReasoningModeId::Medium, Required),
    user("gemini-3.5-flash-lite", ReasoningModeId::Medium, Required),
    tool("gemini-3.5-flash-lite", ReasoningModeId::Medium, Required),
];
const MISTRAL: &[ModelPolicy] = &[
    user("mistral-small-2603", ReasoningModeId::High, Required),
    tool("mistral-small-2603", ReasoningModeId::High, Required),
];
const CEREBRAS: &[ModelPolicy] = &[
    user("zai-glm-4.7", ReasoningModeId::Auto, Optional),
    tool("zai-glm-4.7", ReasoningModeId::Auto, Optional),
];
const OPENROUTER: &[ModelPolicy] = &[
    user("moonshotai/kimi-k2.5", ReasoningModeId::Medium, Required),
    tool("moonshotai/kimi-k2.5", ReasoningModeId::Medium, Required),
];
const OPENAI: &[ModelPolicy] = &[
    user("gpt-5.6-luna", ReasoningModeId::Medium, Required),
    tool("gpt-5.6-luna", ReasoningModeId::Medium, Required),
    user("gpt-5.6-terra", ReasoningModeId::Medium, Required),
    tool("gpt-5.6-terra", ReasoningModeId::Medium, Required),
];
const DEEPSEEK: &[ModelPolicy] = &[
    user("deepseek-v4-flash", ReasoningModeId::High, Forbidden),
    tool("deepseek-v4-flash", ReasoningModeId::High, Required),
];
const XAI: &[ModelPolicy] = &[
    user("grok-4.6", ReasoningModeId::High, Forbidden),
    tool("grok-4.6", ReasoningModeId::High, Forbidden),
];
const MOONSHOT: &[ModelPolicy] = &[
    user("kimi-k2.7-code", ReasoningModeId::Auto, Required),
    tool("kimi-k2.7-code", ReasoningModeId::Auto, Required),
];
const MOONSHOT_OAUTH: &[ModelPolicy] = &[
    user("kimi-for-coding", ReasoningModeId::Auto, Required),
    tool("kimi-for-coding", ReasoningModeId::Auto, Required),
];
pub(super) const ACTIVE_ROUTES: &[RouteContract] = &[
    route(
        RouteId::Ollama,
        ContractId::OllamaNativeV1,
        AdapterId::OllamaNative,
        OLLAMA,
    ),
    route(
        RouteId::Google,
        ContractId::GeminiCompatV1,
        AdapterId::GeminiParts,
        GOOGLE,
    ),
    route(
        RouteId::Mistral,
        ContractId::MistralChunksV1,
        AdapterId::MistralChunks,
        MISTRAL,
    ),
    route(
        RouteId::Cerebras,
        ContractId::CerebrasChatV1,
        AdapterId::CerebrasReasoning,
        CEREBRAS,
    ),
    route(
        RouteId::OpenRouter,
        ContractId::OpenRouterDetailsV1,
        AdapterId::OpenRouterDetails,
        OPENROUTER,
    ),
    route(
        RouteId::OpenAi,
        ContractId::OpenAiResponsesV1,
        AdapterId::ResponsesLocal,
        OPENAI,
    ),
    route(
        RouteId::DeepSeek,
        ContractId::DeepSeekChatV1,
        AdapterId::ChatReasoning,
        DEEPSEEK,
    ),
    route(
        RouteId::Xai,
        ContractId::XaiResponsesV1,
        AdapterId::ResponsesLocal,
        XAI,
    ),
    route(
        RouteId::XaiOauth,
        ContractId::XaiResponsesV1,
        AdapterId::ResponsesLocal,
        XAI_OAUTH,
    ),
    route(
        RouteId::Moonshot,
        ContractId::KimiChatV1,
        AdapterId::ChatReasoning,
        MOONSHOT,
    ),
    route(
        RouteId::MoonshotOauth,
        ContractId::KimiChatV1,
        AdapterId::ChatReasoning,
        MOONSHOT_OAUTH,
    ),
    route(
        RouteId::Zai,
        ContractId::ZaiChatV1,
        AdapterId::ChatReasoning,
        ZAI,
    ),
    route(
        RouteId::CodexOauth,
        ContractId::CodexResponsesV1,
        AdapterId::ResponsesLocal,
        CODEX,
    ),
];

const fn route(
    route_id: RouteId,
    contract_id: ContractId,
    adapter: AdapterId,
    models: &'static [ModelPolicy],
) -> RouteContract {
    RouteContract {
        route_id,
        contract_id,
        adapter,
        models,
    }
}
