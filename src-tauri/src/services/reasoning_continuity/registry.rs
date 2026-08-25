use super::contract::{ContractId, ReplayTarget, RouteId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplayRequirement {
    Required,
    Optional,
    Forbidden,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivationState {
    Disabled,
    #[allow(
        dead_code,
        reason = "fixtures are promoted only after the Task 19 gate"
    )]
    FixtureValidated,
    LiveValidated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdapterId {
    OllamaNative,
    GeminiParts,
    MistralChunks,
    CerebrasReasoning,
    OpenRouterDetails,
    ResponsesLocal,
    ChatReasoning,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModelPolicy {
    pub model_id: &'static str,
    pub requirement: ReplayRequirement,
    pub activation: ActivationState,
    pub fixture_id: Option<&'static str>,
    pub fixture_date: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RouteContract {
    pub route_id: RouteId,
    pub contract_id: ContractId,
    pub adapter: AdapterId,
    pub models: &'static [ModelPolicy],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReplayPolicy {
    pub contract_id: Option<ContractId>,
    pub adapter: Option<AdapterId>,
    pub requirement: ReplayRequirement,
    pub activation: ActivationState,
}

const fn disabled(model_id: &'static str, requirement: ReplayRequirement) -> ModelPolicy {
    ModelPolicy {
        model_id,
        requirement,
        activation: ActivationState::Disabled,
        fixture_id: None,
        fixture_date: None,
    }
}
const OLLAMA_MODELS: &[ModelPolicy] = &[
    disabled("gemma4:e2b-it-q4_K_M", ReplayRequirement::Optional),
    disabled("qwen3.5:4b", ReplayRequirement::Optional),
    disabled("deepseek-r1:latest", ReplayRequirement::Optional),
];
const GOOGLE_MODELS: &[ModelPolicy] = &[
    disabled("gemini-3.7-flash", ReplayRequirement::Required),
    disabled("gemini-3.5-flash", ReplayRequirement::Required),
    disabled("gemini-3.5-flash-lite", ReplayRequirement::Required),
];
const MISTRAL_MODELS: &[ModelPolicy] = &[
    disabled("mistral-small-2603", ReplayRequirement::Required),
    disabled("ministral-14b-2512", ReplayRequirement::Optional),
];
const CEREBRAS_MODELS: &[ModelPolicy] = &[disabled("zai-glm-4.7", ReplayRequirement::Optional)];
const OPENROUTER_MODELS: &[ModelPolicy] = &[
    disabled("moonshotai/kimi-k2.5", ReplayRequirement::Required),
    disabled("stealth/ox-alpha", ReplayRequirement::Optional),
];
const OPENAI_MODELS: &[ModelPolicy] = &[
    disabled("gpt-5.6-luna", ReplayRequirement::Required),
    disabled("gpt-5.6-terra", ReplayRequirement::Required),
];
const DEEPSEEK_MODELS: &[ModelPolicy] =
    &[disabled("deepseek-v4-flash", ReplayRequirement::Required)];
const XAI_MODELS: &[ModelPolicy] = &[disabled("grok-4.6", ReplayRequirement::Forbidden)];
const XAI_OAUTH_MODELS: &[ModelPolicy] = &[disabled("grok-4.6", ReplayRequirement::Required)];
const MOONSHOT_MODELS: &[ModelPolicy] = &[disabled("kimi-k2.7-code", ReplayRequirement::Required)];
const MOONSHOT_OAUTH_MODELS: &[ModelPolicy] =
    &[disabled("kimi-for-coding", ReplayRequirement::Required)];
const ZAI_MODELS: &[ModelPolicy] = &[
    disabled("glm-4.5-flash", ReplayRequirement::Optional),
    disabled("glm-5.3", ReplayRequirement::Optional),
];
const CODEX_MODELS: &[ModelPolicy] = &[disabled("gpt-5.6-luna", ReplayRequirement::Required)];
const ACTIVE_ROUTES: &[RouteContract] = &[
    route(
        RouteId::Ollama,
        ContractId::OllamaNativeV1,
        AdapterId::OllamaNative,
        OLLAMA_MODELS,
    ),
    route(
        RouteId::Google,
        ContractId::GeminiCompatV1,
        AdapterId::GeminiParts,
        GOOGLE_MODELS,
    ),
    route(
        RouteId::Mistral,
        ContractId::MistralChunksV1,
        AdapterId::MistralChunks,
        MISTRAL_MODELS,
    ),
    route(
        RouteId::Cerebras,
        ContractId::CerebrasChatV1,
        AdapterId::CerebrasReasoning,
        CEREBRAS_MODELS,
    ),
    route(
        RouteId::OpenRouter,
        ContractId::OpenRouterDetailsV1,
        AdapterId::OpenRouterDetails,
        OPENROUTER_MODELS,
    ),
    route(
        RouteId::OpenAi,
        ContractId::OpenAiResponsesV1,
        AdapterId::ResponsesLocal,
        OPENAI_MODELS,
    ),
    route(
        RouteId::DeepSeek,
        ContractId::DeepSeekChatV1,
        AdapterId::ChatReasoning,
        DEEPSEEK_MODELS,
    ),
    route(
        RouteId::Xai,
        ContractId::XaiResponsesV1,
        AdapterId::ResponsesLocal,
        XAI_MODELS,
    ),
    route(
        RouteId::XaiOauth,
        ContractId::XaiResponsesV1,
        AdapterId::ResponsesLocal,
        XAI_OAUTH_MODELS,
    ),
    route(
        RouteId::Moonshot,
        ContractId::KimiChatV1,
        AdapterId::ChatReasoning,
        MOONSHOT_MODELS,
    ),
    route(
        RouteId::MoonshotOauth,
        ContractId::KimiChatV1,
        AdapterId::ChatReasoning,
        MOONSHOT_OAUTH_MODELS,
    ),
    route(
        RouteId::Zai,
        ContractId::ZaiChatV1,
        AdapterId::ChatReasoning,
        ZAI_MODELS,
    ),
    route(
        RouteId::CodexOauth,
        ContractId::CodexResponsesV1,
        AdapterId::ResponsesLocal,
        CODEX_MODELS,
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

pub fn active_routes() -> &'static [RouteContract] {
    ACTIVE_ROUTES
}

pub fn route_contract(route_id: RouteId) -> Option<ContractId> {
    ACTIVE_ROUTES
        .iter()
        .find(|entry| entry.route_id == route_id)
        .map(|entry| entry.contract_id)
}

pub fn replay_policy(target: &ReplayTarget) -> Option<ReplayPolicy> {
    if target.route_id == RouteId::Groq {
        return Some(ReplayPolicy {
            contract_id: None,
            adapter: None,
            requirement: ReplayRequirement::Forbidden,
            activation: ActivationState::Disabled,
        });
    }
    let route = ACTIVE_ROUTES
        .iter()
        .find(|entry| entry.route_id == target.route_id)?;
    let model = route
        .models
        .iter()
        .find(|model| model.model_id == target.model_id)?;
    Some(ReplayPolicy {
        contract_id: Some(route.contract_id),
        adapter: Some(route.adapter),
        requirement: model.requirement,
        activation: model.activation,
    })
}
