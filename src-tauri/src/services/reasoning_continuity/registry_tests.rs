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
fn inventory_has_exactly_eleven_contracts_and_thirteen_closed_routes() {
    assert_eq!(ContractId::ALL.len(), 11);
    assert_eq!(RouteId::ALL.len(), 13);
    assert_eq!(active_routes().len(), 13);
}

#[test]
fn every_live_activation_has_one_checked_in_capture_and_replay_proof() {
    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("test-fixtures/reasoning-reports");
    let expected = active_routes()
        .iter()
        .flat_map(|route| route.models)
        .filter(|policy| policy.activation == ActivationState::LiveValidated)
        .map(|policy| {
            let fixture_id = policy.fixture_id.expect("live fixture id");
            assert!(fixture_id.ends_with(policy.fixture_date.expect("live fixture date")));
            fixture_id.to_string()
        })
        .collect::<std::collections::BTreeSet<_>>();
    let actual = std::fs::read_dir(&root)
        .expect("checked-in reasoning fixture reports")
        .map(|entry| {
            entry
                .expect("fixture report entry")
                .file_name()
                .to_str()
                .and_then(|name| name.strip_suffix(".json"))
                .map(str::to_owned)
                .expect("canonical fixture report name")
        })
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        actual, expected,
        "activation and checked-in proofs diverged"
    );

    for fixture_id in expected {
        let bytes =
            std::fs::read(root.join(format!("{fixture_id}.json"))).expect("read fixture proof");
        assert!(bytes.len() <= 256 * 1024);
        let report: serde_json::Value =
            serde_json::from_slice(&bytes).expect("parse fixture proof");
        assert_eq!(report["fixture_id"], fixture_id);
        let scenarios = report["scenarios"].as_array().expect("fixture scenarios");
        for requirement in ["capture_and_persist", "replay_and_continue"] {
            assert!(scenarios.iter().any(|scenario| {
                scenario["requirement"] == requirement && scenario["status"] == "passe"
            }));
        }
    }

    for policy in active_routes()
        .iter()
        .flat_map(|route| route.models)
        .filter(|policy| policy.activation == ActivationState::LiveValidated)
    {
        let fixture_id = policy.fixture_id.expect("live fixture id");
        let bytes = std::fs::read(root.join(format!("{fixture_id}.json")))
            .expect("read exact-mode fixture proof");
        let report: serde_json::Value =
            serde_json::from_slice(&bytes).expect("parse exact-mode fixture proof");
        assert_eq!(
            report["reasoning_mode"],
            serde_json::to_value(policy.reasoning_mode).unwrap(),
            "{fixture_id} does not prove its activated reasoning mode"
        );
    }
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
    assert_eq!(RouteId::from_provider_id("groq"), None);
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
}

#[test]
fn r01_unknown_model_fails_closed() {
    let scope = CredentialScope::authenticated("fixture-scope").unwrap();
    let unknown_model = ReplayTarget {
        route_id: RouteId::OpenRouter,
        model_id: "unknown/model".into(),
        credential_scope: scope.clone(),
        reasoning_mode: ReasoningModeId::High,
        continuation_use: ContinuationUse::UserContinuation,
    };
    assert!(replay_policy(&unknown_model).is_none());
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
    assert_eq!(user.requirement(), ReplayRequirement::Required);
    assert_eq!(user.activation(), ActivationState::LiveValidated);

    let tool = replay_policy_from_routes(
        ROUTES,
        &target(ReasoningModeId::High, ContinuationUse::ToolContinuation),
    )
    .unwrap();
    assert_eq!(tool.requirement(), ReplayRequirement::Required);
    assert_eq!(tool.activation(), ActivationState::Disabled);

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
fn deepseek_replay_is_required_for_every_supported_effort_and_continuation() {
    let scope = CredentialScope::authenticated("fixture-scope").unwrap();
    let target = |reasoning_mode, continuation_use| ReplayTarget {
        route_id: RouteId::DeepSeek,
        model_id: "deepseek-v4-flash".into(),
        credential_scope: scope.clone(),
        reasoning_mode,
        continuation_use,
    };

    for reasoning_mode in [
        ReasoningModeId::Low,
        ReasoningModeId::High,
        ReasoningModeId::Max,
    ] {
        for continuation_use in [
            ContinuationUse::UserContinuation,
            ContinuationUse::ToolContinuation,
        ] {
            let policy = replay_policy(&target(reasoning_mode, continuation_use)).unwrap();
            assert_eq!(policy.requirement(), ReplayRequirement::Required);
            assert_eq!(policy.activation(), ActivationState::LiveValidated);
        }
    }
    assert!(replay_policy(&ReplayTarget {
        reasoning_mode: ReasoningModeId::Off,
        ..target(ReasoningModeId::High, ContinuationUse::ToolContinuation)
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
                ..target(ReasoningModeId::High, ContinuationUse::ToolContinuation)
            }
        ),
        ReplayDecision::Blocked(BlockReason::ProvenanceMismatch)
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
fn only_exact_live_fixture_pairs_are_activated() {
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
    let expected = [
        (
            RouteId::Ollama,
            "gemma4:e2b-it-q4_K_M",
            ReasoningModeId::Auto,
            ReplayRequirement::Optional,
            "ollama-local-gemma4-e2b-it-q4-k-m-local-2026-08-26",
        ),
        (
            RouteId::Ollama,
            "qwen3.5:4b",
            ReasoningModeId::Auto,
            ReplayRequirement::Optional,
            "ollama-local-qwen3-5-4b-local-2026-08-26",
        ),
        (
            RouteId::Google,
            "gemini-3.5-flash",
            ReasoningModeId::Medium,
            ReplayRequirement::Required,
            "google-api-gemini-3-5-flash-france-2026-08-26",
        ),
        (
            RouteId::Cerebras,
            "gpt-oss-120b",
            ReasoningModeId::High,
            ReplayRequirement::Required,
            "cerebras-api-gpt-oss-120b-france-2026-08-26",
        ),
        (
            RouteId::Mistral,
            "mistral-small-2603",
            ReasoningModeId::High,
            ReplayRequirement::Required,
            "mistral-api-mistral-small-2603-france-2026-08-26",
        ),
        (
            RouteId::OpenRouter,
            "moonshotai/kimi-k2.5",
            ReasoningModeId::Medium,
            ReplayRequirement::Required,
            "openrouter-api-moonshotai-kimi-k2-5-france-2026-08-26",
        ),
        (
            RouteId::OpenAi,
            "gpt-5.6-luna",
            ReasoningModeId::Medium,
            ReplayRequirement::Required,
            "openai-api-gpt-5-6-luna-france-2026-08-26",
        ),
        (
            RouteId::DeepSeek,
            "deepseek-v4-flash",
            ReasoningModeId::Low,
            ReplayRequirement::Required,
            "deepseek-api-deepseek-v4-flash-low-france-2026-08-29",
        ),
        (
            RouteId::DeepSeek,
            "deepseek-v4-flash",
            ReasoningModeId::High,
            ReplayRequirement::Required,
            "deepseek-api-deepseek-v4-flash-high-france-2026-08-29",
        ),
        (
            RouteId::DeepSeek,
            "deepseek-v4-flash",
            ReasoningModeId::Max,
            ReplayRequirement::Required,
            "deepseek-api-deepseek-v4-flash-max-france-2026-08-29",
        ),
        (
            RouteId::Xai,
            "grok-4.6",
            ReasoningModeId::High,
            ReplayRequirement::Required,
            "xai-api-grok-4-6-france-2026-08-26",
        ),
        (
            RouteId::CodexOauth,
            "gpt-5.6-luna",
            ReasoningModeId::Medium,
            ReplayRequirement::Required,
            "codex-oauth-gpt-5-6-luna-local-2026-08-26",
        ),
        (
            RouteId::XaiOauth,
            "grok-4.6",
            ReasoningModeId::High,
            ReplayRequirement::Required,
            "xai-oauth-grok-4-6-local-2026-08-26",
        ),
        (
            RouteId::Moonshot,
            "kimi-k2.7-code",
            ReasoningModeId::Auto,
            ReplayRequirement::Required,
            "moonshot-api-kimi-k2-7-code-france-2026-08-26",
        ),
        (
            RouteId::Zai,
            "glm-4.5-flash",
            ReasoningModeId::Auto,
            ReplayRequirement::Optional,
            "zai-api-glm-4-5-flash-local-2026-08-26",
        ),
    ];
    let expected_count = expected.len() * 2;
    assert_eq!(live.len(), expected_count);
    for (route, model) in &live {
        assert!(expected.iter().any(|entry| {
            *route == entry.0
                && model.model_id == entry.1
                && model.reasoning_mode == entry.2
                && model.requirement == entry.3
                && model.fixture_id == Some(entry.4)
        }));
        assert!(matches!(
            model.continuation_use,
            ContinuationUse::UserContinuation | ContinuationUse::ToolContinuation
        ));
        let expected_date = if *route == RouteId::DeepSeek {
            "2026-08-29"
        } else {
            "2026-08-26"
        };
        assert_eq!(model.fixture_date, Some(expected_date));
    }
    for entry in expected {
        for continuation_use in [
            ContinuationUse::UserContinuation,
            ContinuationUse::ToolContinuation,
        ] {
            assert_eq!(
                live.iter()
                    .filter(|(route, model)| {
                        *route == entry.0
                            && model.model_id == entry.1
                            && model.reasoning_mode == entry.2
                            && model.continuation_use == continuation_use
                    })
                    .count(),
                1
            );
        }
    }
}
