use super::*;
use crate::services::agent_local::types_ollama::{ToolCallFunction, ToolCallOllama};
use crate::services::reasoning_continuity::contract::{
    ContinuationTarget, CredentialScope, ReasoningModeId, RouteId,
};
use crate::services::reasoning_continuity::envelope::{CompletionState, ReasoningSource};
use crate::services::reasoning_continuity::registry::{ActivationState, ReplayRequirement};
use serde_json::json;

fn target(route_id: RouteId, model_id: &str, continuation_use: ContinuationUse) -> ReplayTarget {
    ReplayTarget {
        route_id,
        model_id: model_id.into(),
        credential_scope: if route_id == RouteId::Ollama {
            CredentialScope::local_uncredentialed()
        } else {
            CredentialScope::authenticated("fixture-scope").unwrap()
        },
        reasoning_mode: ReasoningModeId::Auto,
        continuation_use,
    }
}

fn envelope(
    target: &ReplayTarget,
    contract_id: ContractId,
    continuation: ContinuationState,
) -> ReasoningEnvelope {
    ReasoningEnvelope::new(
        contract_id,
        ReasoningSource {
            route_id: target.route_id,
            model_id: target.model_id.clone(),
            credential_scope: target.credential_scope.clone(),
            reasoning_mode: target.reasoning_mode,
        },
        CompletionState::Complete,
        continuation,
        Vec::new(),
    )
}

fn policy(
    contract_id: ContractId,
    adapter: AdapterId,
    activation: ActivationState,
) -> ReplayPolicy {
    ReplayPolicy::for_test(
        Some(contract_id),
        Some(adapter),
        ReplayRequirement::Required,
        activation,
    )
}

fn assistant(envelope: ReasoningEnvelope, with_tool: bool) -> ChatMessage {
    ChatMessage::assistant(
        String::new(),
        None,
        Some(envelope),
        None,
        with_tool.then(|| {
            vec![ToolCallOllama {
                id: Some("call_1".into()),
                extra_content: Some(json!({"legacy": "ignored"})),
                function: ToolCallFunction {
                    name: "lookup".into(),
                    arguments: json!({}),
                },
            }]
        }),
    )
}

#[test]
fn disabled_policy_cannot_construct_a_replay_approval() {
    let target = target(
        RouteId::Moonshot,
        "kimi-k2.7-code",
        ContinuationUse::ToolContinuation,
    );
    let envelope = envelope(
        &target,
        ContractId::KimiChatV1,
        ContinuationState::ChatReasoning {
            reasoning_content: String::new(),
        },
    );

    let result = approved(
        ReplayDecision::Allowed,
        policy(
            ContractId::KimiChatV1,
            AdapterId::ChatReasoning,
            ActivationState::Disabled,
        ),
        &envelope,
        &target,
    );

    assert_eq!(result.unwrap_err(), ReplayApplyError::Blocked);
}

#[test]
fn deepseek_replay_requires_the_tool_chain() {
    let target = target(
        RouteId::DeepSeek,
        "deepseek-v4-flash",
        ContinuationUse::ToolContinuation,
    );
    let envelope = envelope(
        &target,
        ContractId::DeepSeekChatV1,
        ContinuationState::ChatReasoning {
            reasoning_content: "opaque".into(),
        },
    );
    let approval = approved(
        ReplayDecision::Allowed,
        policy(
            ContractId::DeepSeekChatV1,
            AdapterId::ChatReasoning,
            ActivationState::LiveValidated,
        ),
        &envelope,
        &target,
    )
    .unwrap();
    let messages = [assistant(envelope.clone(), true)];
    let mut payload = vec![json!({"role": "assistant", "tool_calls": []})];

    apply_chat_continuity(&messages, &approval, &mut payload).unwrap();

    assert_eq!(payload[0]["reasoning_content"], "opaque");
}

#[test]
fn deepseek_keeps_an_empty_reasoning_field_in_the_tool_chain() {
    let target = target(
        RouteId::DeepSeek,
        "deepseek-v4-flash",
        ContinuationUse::ToolContinuation,
    );
    let envelope = envelope(
        &target,
        ContractId::DeepSeekChatV1,
        ContinuationState::ChatReasoning {
            reasoning_content: String::new(),
        },
    );
    let approval = approved(
        ReplayDecision::Allowed,
        policy(
            ContractId::DeepSeekChatV1,
            AdapterId::ChatReasoning,
            ActivationState::LiveValidated,
        ),
        &envelope,
        &target,
    )
    .unwrap();
    let messages = [assistant(envelope.clone(), true)];
    let mut payload = vec![json!({"role": "assistant", "tool_calls": []})];

    apply_chat_continuity(&messages, &approval, &mut payload).unwrap();

    assert_eq!(payload[0]["reasoning_content"], "");
}

#[test]
fn kimi_keeps_an_empty_reasoning_field_only_for_its_matching_provenance() {
    let target = target(
        RouteId::MoonshotOauth,
        "kimi-for-coding",
        ContinuationUse::ToolContinuation,
    );
    let envelope = envelope(
        &target,
        ContractId::KimiChatV1,
        ContinuationState::ChatReasoning {
            reasoning_content: String::new(),
        },
    );
    let approval = approved(
        ReplayDecision::Allowed,
        policy(
            ContractId::KimiChatV1,
            AdapterId::ChatReasoning,
            ActivationState::LiveValidated,
        ),
        &envelope,
        &target,
    )
    .unwrap();
    let messages = [assistant(envelope.clone(), false)];
    let mut payload = vec![json!({"role": "assistant"})];

    apply_chat_continuity(&messages, &approval, &mut payload).unwrap();

    assert_eq!(payload[0]["reasoning_content"], "");
}

#[test]
fn zai_opt_in_is_a_top_level_thinking_contract() {
    let target = target(RouteId::Zai, "glm-5.3", ContinuationUse::ToolContinuation);
    let envelope = envelope(
        &target,
        ContractId::ZaiChatV1,
        ContinuationState::ChatReasoning {
            reasoning_content: "opaque".into(),
        },
    );
    let approval = approved(
        ReplayDecision::Allowed,
        policy(
            ContractId::ZaiChatV1,
            AdapterId::ChatReasoning,
            ActivationState::LiveValidated,
        ),
        &envelope,
        &target,
    )
    .unwrap();
    let mut payload = json!({"messages": []});

    apply_chat_payload_continuity(&approval, &mut payload).unwrap();

    assert_eq!(
        payload["thinking"],
        json!({"type": "enabled", "clear_thinking": false})
    );
}

#[test]
fn zai_flash_replay_keeps_clear_thinking_disabled() {
    let target = target(
        RouteId::Zai,
        "glm-5.3-flash",
        ContinuationUse::ToolContinuation,
    );
    let envelope = envelope(
        &target,
        ContractId::ZaiChatV1,
        ContinuationState::ChatReasoning {
            reasoning_content: "opaque".into(),
        },
    );
    let approval = approved(
        ReplayDecision::Allowed,
        policy(
            ContractId::ZaiChatV1,
            AdapterId::ChatReasoning,
            ActivationState::LiveValidated,
        ),
        &envelope,
        &target,
    )
    .unwrap();
    let mut payload = json!({"messages": []});

    apply_chat_payload_continuity(&approval, &mut payload).unwrap();

    assert_eq!(
        payload["thinking"],
        json!({"type": "enabled", "clear_thinking": false})
    );
}

#[test]
fn native_adapters_preserve_their_opaque_shapes() {
    let cases = [
        (
            RouteId::Mistral,
            "mistral-small-2603",
            ContractId::MistralChunksV1,
            AdapterId::MistralChunks,
            ContinuationState::MistralChunks {
                chunks: vec![json!({"type": "reasoning", "text": "chunk"})],
            },
            "content",
        ),
        (
            RouteId::OpenRouter,
            "moonshotai/kimi-k2.5",
            ContractId::OpenRouterDetailsV1,
            AdapterId::OpenRouterDetails,
            ContinuationState::OpenRouterDetails {
                details: vec![json!({"type": "reasoning.text", "text": "detail"})],
            },
            "reasoning_details",
        ),
    ];
    for (route, model, contract, adapter, continuation, field) in cases {
        let target = target(route, model, ContinuationUse::ToolContinuation);
        let envelope = envelope(&target, contract, continuation.clone());
        let approval = approved(
            ReplayDecision::Allowed,
            policy(contract, adapter, ActivationState::LiveValidated),
            &envelope,
            &target,
        )
        .unwrap();
        let messages = [assistant(envelope.clone(), false)];
        let mut payload = vec![json!({"role": "assistant"})];

        apply_chat_continuity(&messages, &approval, &mut payload).unwrap();

        let expected = match continuation {
            ContinuationState::MistralChunks { chunks: parts }
            | ContinuationState::OpenRouterDetails { details: parts } => json!(parts),
            _ => unreachable!(),
        };
        assert_eq!(payload[0][field], expected);
    }
}

#[test]
fn ollama_and_responses_adapters_keep_their_distinct_wires() {
    let ollama_target = target(
        RouteId::Ollama,
        "qwen3.5:4b",
        ContinuationUse::ToolContinuation,
    );
    let ollama_envelope = envelope(
        &ollama_target,
        ContractId::OllamaNativeV1,
        ContinuationState::OllamaNative {
            thinking: "native".into(),
        },
    );
    let ollama_approval = approved(
        ReplayDecision::Allowed,
        policy(
            ContractId::OllamaNativeV1,
            AdapterId::OllamaNative,
            ActivationState::LiveValidated,
        ),
        &ollama_envelope,
        &ollama_target,
    )
    .unwrap();
    let ollama_messages = [assistant(ollama_envelope.clone(), false)];
    let mut ollama_payload = vec![json!({"role": "assistant"})];
    apply_ollama_continuity(&ollama_messages, &ollama_approval, &mut ollama_payload).unwrap();
    assert_eq!(ollama_payload[0]["thinking"], "native");

    let target = target(
        RouteId::CodexOauth,
        "gpt-5.6-luna",
        ContinuationUse::ToolContinuation,
    );
    let envelope = envelope(
        &target,
        ContractId::CodexResponsesV1,
        ContinuationState::ResponsesLocal {
            items: vec![json!({"type": "function_call", "call_id": "call_1"})],
        },
    );
    let approval = approved(
        ReplayDecision::Allowed,
        policy(
            ContractId::CodexResponsesV1,
            AdapterId::ResponsesLocal,
            ActivationState::LiveValidated,
        ),
        &envelope,
        &target,
    )
    .unwrap();
    let messages = [assistant(envelope.clone(), true)];
    let mut input = Vec::new();
    apply_responses_continuity(&messages, &approval, &mut input).unwrap();
    assert_eq!(input[0]["call_id"], "call_1");
    assert!(input[0].get("extra_content").is_none());
}

#[cfg(debug_assertions)]
#[test]
fn codex_astra_fixture_candidate_replays_persisted_response_items() {
    let target = ReplayTarget {
        route_id: RouteId::CodexOauth,
        model_id: "gpt-6-astra".into(),
        credential_scope: CredentialScope::authenticated("codex-scope").unwrap(),
        reasoning_mode: ReasoningModeId::High,
        continuation_use: ContinuationUse::ToolContinuation,
    };
    let envelope = envelope(
        &target,
        ContractId::CodexResponsesV1,
        ContinuationState::ResponsesLocal {
            items: vec![
                json!({"type": "reasoning", "encrypted_content": "codex-opaque-Δ"}),
                json!({"type": "function_call", "call_id": "call-codex"}),
            ],
        },
    );
    let reloaded: ReasoningEnvelope = serde_json::from_slice(
        &serde_json::to_vec(&envelope).expect("persisted Codex Astra envelope"),
    )
    .expect("reloaded Codex Astra envelope");
    let candidate = ContinuationTarget::FixtureCandidate(target.clone());
    let approval = approval_for_target(&candidate, &reloaded).expect("Codex fixture approval");
    let messages = [
        assistant(reloaded.clone(), true),
        ChatMessage::user("continue".into()),
    ];
    let mut input = Vec::new();

    apply_responses_continuity(&messages, &approval, &mut input).expect("Codex fixture replay");

    assert_eq!(input[0]["encrypted_content"], "codex-opaque-Δ");
    assert_eq!(input[1]["call_id"], "call-codex");
}
