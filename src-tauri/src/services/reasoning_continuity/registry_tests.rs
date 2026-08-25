use super::contract::{ContractId, CredentialScope, ReasoningModeId, ReplayTarget, RouteId};
use super::eligibility::{decide, BlockReason, ReplayDecision};
use super::envelope::{CompletionState, ContinuationState, ReasoningEnvelope, ReasoningSource};
use super::registry::{active_routes, replay_policy, route_contract, ReplayRequirement};

#[test]
fn inventory_has_exactly_eleven_contracts_and_fourteen_closed_routes() {
    assert_eq!(ContractId::ALL.len(), 11);
    assert_eq!(RouteId::ALL.len(), 14);
    assert_eq!(active_routes().len(), 13);
}

#[test]
fn closed_identifiers_serialize_to_the_exact_normative_wire_values() {
    let contracts = [
        "ollama-native-v1",
        "gemini-compat-v1",
        "mistral-chunks-v1",
        "cerebras-chat-v1",
        "openrouter-details-v1",
        "openai-responses-v1",
        "deepseek-chat-v1",
        "xai-responses-v1",
        "kimi-chat-v1",
        "zai-chat-v1",
        "codex-responses-v1",
    ];
    let routes = [
        "ollama",
        "google",
        "mistral",
        "cerebras",
        "openrouter",
        "openai",
        "deepseek",
        "xai",
        "xai-oauth",
        "moonshot",
        "moonshot-oauth",
        "zai",
        "codex-oauth",
        "groq",
    ];

    for (value, expected) in ContractId::ALL.iter().zip(contracts) {
        assert_eq!(serde_json::to_value(value).unwrap(), expected);
    }
    for (value, expected) in RouteId::ALL.iter().zip(routes) {
        assert_eq!(serde_json::to_value(value).unwrap(), expected);
    }
}

#[test]
fn every_scoped_route_maps_to_the_normative_contract() {
    let expected = [
        (RouteId::Ollama, ContractId::OllamaNativeV1),
        (RouteId::Google, ContractId::GeminiCompatV1),
        (RouteId::Mistral, ContractId::MistralChunksV1),
        (RouteId::Cerebras, ContractId::CerebrasChatV1),
        (RouteId::OpenRouter, ContractId::OpenRouterDetailsV1),
        (RouteId::OpenAi, ContractId::OpenAiResponsesV1),
        (RouteId::DeepSeek, ContractId::DeepSeekChatV1),
        (RouteId::Xai, ContractId::XaiResponsesV1),
        (RouteId::XaiOauth, ContractId::XaiResponsesV1),
        (RouteId::Moonshot, ContractId::KimiChatV1),
        (RouteId::MoonshotOauth, ContractId::KimiChatV1),
        (RouteId::Zai, ContractId::ZaiChatV1),
        (RouteId::CodexOauth, ContractId::CodexResponsesV1),
    ];
    assert_eq!(active_routes().len(), expected.len());
    for (route, contract) in expected {
        assert_eq!(route_contract(route), Some(contract));
    }
    assert_eq!(route_contract(RouteId::Groq), None);
}

#[test]
fn r01_unknown_model_and_excluded_route_fail_closed() {
    let scope = CredentialScope::authenticated("fixture-scope").unwrap();
    let unknown_model = ReplayTarget {
        route_id: RouteId::OpenRouter,
        model_id: "unknown/model".into(),
        credential_scope: scope.clone(),
        reasoning_mode: ReasoningModeId::High,
    };
    assert!(replay_policy(&unknown_model).is_none());

    let groq = ReplayTarget {
        route_id: RouteId::Groq,
        model_id: "any".into(),
        credential_scope: scope,
        reasoning_mode: ReasoningModeId::Auto,
    };
    assert_eq!(
        replay_policy(&groq).unwrap().requirement,
        ReplayRequirement::Forbidden
    );
    assert_eq!(
        replay_policy(&groq).unwrap().activation,
        super::registry::ActivationState::Disabled
    );
}

#[test]
fn r01_unknown_target_is_blocked_before_any_replay() {
    let scope = CredentialScope::authenticated("fixture-scope").unwrap();
    let envelope = ReasoningEnvelope::new(
        ContractId::OpenRouterDetailsV1,
        ReasoningSource {
            route_id: RouteId::OpenRouter,
            model_id: "moonshotai/kimi-k2.5".into(),
            credential_scope: scope.clone(),
            reasoning_mode: ReasoningModeId::High,
        },
        CompletionState::Complete,
        ContinuationState::OpenRouterDetails {
            details: Vec::new(),
        },
        Vec::new(),
    );
    let unknown = ReplayTarget {
        route_id: RouteId::OpenRouter,
        model_id: "unknown/model".into(),
        credential_scope: scope,
        reasoning_mode: ReasoningModeId::High,
    };

    assert_eq!(
        decide(&envelope, &unknown),
        ReplayDecision::Blocked(BlockReason::UnknownTarget)
    );
}

#[test]
fn every_declared_model_is_disabled_and_none_is_live_validated() {
    for route in active_routes() {
        assert!(!route.models.is_empty());
        for model in route.models {
            assert_eq!(model.activation, super::registry::ActivationState::Disabled);
            assert!(model.fixture_id.is_none());
            assert!(model.fixture_date.is_none());
        }
    }
}
