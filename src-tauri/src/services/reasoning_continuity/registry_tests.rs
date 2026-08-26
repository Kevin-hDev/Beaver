use super::contract::{
    ContinuationUse, ContractId, CredentialScope, ReasoningModeId, ReplayTarget, RouteId,
};
use super::eligibility::{decide, BlockReason, ReplayDecision};
use super::envelope::{CompletionState, ContinuationState, ReasoningEnvelope, ReasoningSource};
use super::registry::{
    active_routes, replay_policy, replay_policy_from_routes, route_contract, ActivationState,
    AdapterId, ModelPolicy, ReplayRequirement, RouteContract,
};

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
fn provider_route_mapping_has_one_exhaustive_round_trip() {
    for route in RouteId::ALL {
        assert_eq!(RouteId::from_provider_id(route.provider_id()), Some(route));
    }
    assert_eq!(RouteId::from_provider_id("forged"), None);
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
        continuation_use: ContinuationUse::UserContinuation,
    };
    assert!(replay_policy(&unknown_model).is_none());

    let groq = ReplayTarget {
        route_id: RouteId::Groq,
        model_id: "any".into(),
        credential_scope: scope,
        reasoning_mode: ReasoningModeId::Auto,
        continuation_use: ContinuationUse::UserContinuation,
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
        continuation_use: ContinuationUse::UserContinuation,
    };

    assert_eq!(
        decide(&envelope, &unknown),
        ReplayDecision::Blocked(BlockReason::UnknownTarget)
    );
}

#[test]
fn exact_mode_and_continuation_use_are_independent_policy_dimensions() {
    const POLICIES: &[ModelPolicy] = &[
        ModelPolicy {
            model_id: "deepseek-v4-flash",
            reasoning_mode: ReasoningModeId::High,
            continuation_use: ContinuationUse::UserContinuation,
            requirement: ReplayRequirement::Required,
            activation: ActivationState::LiveValidated,
            fixture_id: Some("synthetic-high-user"),
            fixture_date: Some("2026-08-25"),
        },
        ModelPolicy {
            model_id: "deepseek-v4-flash",
            reasoning_mode: ReasoningModeId::High,
            continuation_use: ContinuationUse::ToolContinuation,
            requirement: ReplayRequirement::Required,
            activation: ActivationState::Disabled,
            fixture_id: None,
            fixture_date: None,
        },
    ];
    const ROUTES: &[RouteContract] = &[RouteContract {
        route_id: RouteId::DeepSeek,
        contract_id: ContractId::DeepSeekChatV1,
        adapter: AdapterId::ChatReasoning,
        models: POLICIES,
    }];
    let scope = CredentialScope::authenticated("fixture-scope").unwrap();
    let target = |reasoning_mode, continuation_use| ReplayTarget {
        route_id: RouteId::DeepSeek,
        model_id: "deepseek-v4-flash".into(),
        credential_scope: scope.clone(),
        reasoning_mode,
        continuation_use,
    };

    let user = replay_policy_from_routes(
        ROUTES,
        &target(ReasoningModeId::High, ContinuationUse::UserContinuation),
    )
    .unwrap();
    assert_eq!(user.requirement, ReplayRequirement::Required);
    assert_eq!(user.activation, ActivationState::LiveValidated);

    let tool = replay_policy_from_routes(
        ROUTES,
        &target(ReasoningModeId::High, ContinuationUse::ToolContinuation),
    )
    .unwrap();
    assert_eq!(tool.requirement, ReplayRequirement::Required);
    assert_eq!(tool.activation, ActivationState::Disabled);

    assert!(replay_policy_from_routes(
        ROUTES,
        &target(ReasoningModeId::Low, ContinuationUse::UserContinuation)
    )
    .is_none());
    assert!(replay_policy_from_routes(
        ROUTES,
        &target(ReasoningModeId::Off, ContinuationUse::UserContinuation)
    )
    .is_none());
}

#[test]
fn deepseek_user_and_tool_continuations_have_distinct_requirements() {
    let scope = CredentialScope::authenticated("fixture-scope").unwrap();
    let target = |continuation_use| ReplayTarget {
        route_id: RouteId::DeepSeek,
        model_id: "deepseek-v4-flash".into(),
        credential_scope: scope.clone(),
        reasoning_mode: ReasoningModeId::High,
        continuation_use,
    };

    assert_eq!(
        replay_policy(&target(ContinuationUse::UserContinuation))
            .unwrap()
            .requirement,
        ReplayRequirement::Forbidden
    );
    assert_eq!(
        replay_policy(&target(ContinuationUse::ToolContinuation))
            .unwrap()
            .requirement,
        ReplayRequirement::Required
    );
    assert!(replay_policy(&ReplayTarget {
        reasoning_mode: ReasoningModeId::Off,
        ..target(ContinuationUse::ToolContinuation)
    })
    .is_none());

    let envelope = ReasoningEnvelope::new(
        ContractId::DeepSeekChatV1,
        ReasoningSource {
            route_id: RouteId::DeepSeek,
            model_id: "deepseek-v4-flash".into(),
            credential_scope: scope.clone(),
            reasoning_mode: ReasoningModeId::High,
        },
        CompletionState::Complete,
        ContinuationState::ChatReasoning {
            reasoning_content: "opaque".into(),
        },
        Vec::new(),
    );
    assert_eq!(
        decide(
            &envelope,
            &ReplayTarget {
                reasoning_mode: ReasoningModeId::Low,
                ..target(ContinuationUse::ToolContinuation)
            }
        ),
        ReplayDecision::Blocked(BlockReason::UnknownTarget)
    );
}

#[test]
fn local_scope_is_valid_only_for_ollama() {
    let ollama = ReplayTarget {
        route_id: RouteId::Ollama,
        model_id: "qwen3.5:4b".into(),
        credential_scope: CredentialScope::local_uncredentialed(),
        reasoning_mode: ReasoningModeId::Auto,
        continuation_use: ContinuationUse::UserContinuation,
    };
    assert!(replay_policy(&ollama).is_some());

    let mut authenticated_ollama = ollama.clone();
    authenticated_ollama.credential_scope = CredentialScope::authenticated("scope").unwrap();
    assert!(replay_policy(&authenticated_ollama).is_none());

    let mut local_cloud = ollama;
    local_cloud.route_id = RouteId::DeepSeek;
    local_cloud.model_id = "deepseek-v4-flash".into();
    local_cloud.reasoning_mode = ReasoningModeId::High;
    local_cloud.continuation_use = ContinuationUse::ToolContinuation;
    assert!(replay_policy(&local_cloud).is_none());
}

#[test]
fn only_qwen_auto_user_and_tool_are_live_validated() {
    let mut live = Vec::new();
    for route in active_routes() {
        assert!(!route.models.is_empty());
        for model in route.models {
            assert_ne!(model.reasoning_mode, ReasoningModeId::Off);
            if model.activation == ActivationState::LiveValidated {
                live.push((route.route_id, model));
            } else {
                assert_eq!(model.activation, ActivationState::Disabled);
                assert!(model.fixture_id.is_none());
                assert!(model.fixture_date.is_none());
            }
        }
    }
    assert_eq!(live.len(), 2);
    for (route, model) in live {
        assert_eq!(route, RouteId::Ollama);
        assert_eq!(model.model_id, "qwen3.5:4b");
        assert_eq!(model.reasoning_mode, ReasoningModeId::Auto);
        assert!(matches!(
            model.continuation_use,
            ContinuationUse::UserContinuation | ContinuationUse::ToolContinuation
        ));
        assert_eq!(model.fixture_id, Some("ollama-local-qwen3-5-4b-local-2026-08-26"));
        assert_eq!(model.fixture_date, Some("2026-08-26"));
    }
}
