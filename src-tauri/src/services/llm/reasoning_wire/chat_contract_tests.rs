use super::{build_chat_payload, build_chat_payload_with_evidence};
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::agent_local::types_ollama::{ToolCallFunction, ToolCallOllama};
use crate::services::llm::fast_mode::FastModeRequest;
use crate::services::llm::route;
use crate::services::llm::stream_http::RequestConfig;
use crate::services::llm::stream_test_transport::{ScriptedResponse, StreamScenario};
use crate::services::reasoning_continuity::contract::{
    ContinuationTarget, ContinuationUse, ContractId, CredentialScope, NonReplayTarget,
    ReasoningModeId, ReplayTarget, RouteId,
};
use crate::services::reasoning_continuity::envelope::{
    CompletionState, ContinuationState, ReasoningEnvelope, ReasoningSource,
};
use std::time::Duration;

fn target() -> ContinuationTarget {
    ContinuationTarget::FixtureCandidate(ReplayTarget {
        route_id: RouteId::Moonshot,
        model_id: "kimi-k2.7-code".into(),
        credential_scope: CredentialScope::authenticated("fixture-scope").unwrap(),
        reasoning_mode: ReasoningModeId::Auto,
        continuation_use: ContinuationUse::UserContinuation,
    })
}

fn assistant() -> ChatMessage {
    let replay = target().replay().unwrap().clone();
    let envelope = ReasoningEnvelope::new(
        ContractId::KimiChatV1,
        ReasoningSource::from_target(&replay),
        CompletionState::Complete,
        ContinuationState::ChatReasoning {
            reasoning_content: "opaque-kimi".into(),
        },
        Vec::new(),
    );
    ChatMessage::assistant("answer".into(), None, Some(envelope), None, None)
}

fn replay_target(
    route_id: RouteId,
    model_id: &str,
    reasoning_mode: ReasoningModeId,
    continuation_use: ContinuationUse,
) -> ReplayTarget {
    ReplayTarget {
        route_id,
        model_id: model_id.into(),
        credential_scope: CredentialScope::authenticated("fixture-scope").unwrap(),
        reasoning_mode,
        continuation_use,
    }
}

fn fixture_target(target: ReplayTarget) -> ContinuationTarget {
    ContinuationTarget::FixtureCandidate(target)
}

fn envelope(
    target: &ReplayTarget,
    contract_id: ContractId,
    completion: CompletionState,
    continuation: ContinuationState,
) -> ReasoningEnvelope {
    ReasoningEnvelope::new(
        contract_id,
        ReasoningSource::from_target(target),
        completion,
        continuation,
        Vec::new(),
    )
}

fn payload(
    provider_id: &str,
    model: &str,
    messages: &[ChatMessage],
    target: &ContinuationTarget,
    mode: &str,
) -> Result<serde_json::Value, crate::services::llm::reasoning_wire::replay::ReplayApplyError> {
    let cfg = RequestConfig {
        provider_id,
        model,
        messages,
        tools: &[],
        think: mode != "off",
        reasoning_mode: Some(mode),
        max_tokens: None,
        purpose: crate::services::llm::request_purpose::RequestPurpose::ManualChat,
        session_id: None,
        fast_mode: FastModeRequest::Unsupported,
        continuation_target: Some(target),
    };
    build_chat_payload(&cfg, &route::resolve(provider_id).unwrap(), None)
}

#[test]
fn kimi_reasoning_fixture_payload_places_opaque_reasoning_on_the_same_assistant_message() {
    let messages = [assistant(), ChatMessage::user("continue".into())];
    let target = target();
    let cfg = RequestConfig {
        provider_id: "moonshot",
        model: "kimi-k2.7-code",
        messages: &messages,
        tools: &[],
        think: true,
        reasoning_mode: Some("auto"),
        max_tokens: None,
        purpose: crate::services::llm::request_purpose::RequestPurpose::ManualChat,
        session_id: None,
        fast_mode: FastModeRequest::Unsupported,
        continuation_target: Some(&target),
    };

    let payload = build_chat_payload(&cfg, &route::resolve("moonshot").unwrap(), None)
        .expect("fixture payload");

    assert_eq!(payload["messages"][0]["reasoning_content"], "opaque-kimi");
}

#[test]
fn required_chat_replay_ignores_legacy_assistants_before_the_barrier() {
    let target = target();
    let old = ChatMessage::assistant("legacy answer".into(), None, None, None, None);
    let mut current = ChatMessage::user("continue".into());
    current.continuity_barrier_before = true;
    let messages = [old, current];
    let cfg = RequestConfig {
        provider_id: "moonshot",
        model: "kimi-k2.7-code",
        messages: &messages,
        tools: &[],
        think: true,
        reasoning_mode: Some("auto"),
        max_tokens: None,
        purpose: crate::services::llm::request_purpose::RequestPurpose::ManualChat,
        session_id: None,
        fast_mode: FastModeRequest::Unsupported,
        continuation_target: Some(&target),
    };

    assert!(build_chat_payload(&cfg, &route::resolve("moonshot").unwrap(), None).is_ok());
}

#[test]
fn deepseek_reasoning_tool_continuation_uses_the_tool_contract_even_when_admission_started_as_user()
{
    let target = fixture_target(replay_target(
        RouteId::DeepSeek,
        "deepseek-v4-flash",
        ReasoningModeId::High,
        ContinuationUse::UserContinuation,
    ));
    let replay = target.replay().unwrap();
    let assistant = ChatMessage::assistant(
        String::new(),
        None,
        Some(envelope(
            replay,
            ContractId::DeepSeekChatV1,
            CompletionState::Complete,
            ContinuationState::ChatReasoning {
                reasoning_content: "opaque-tool".into(),
            },
        )),
        None,
        Some(vec![ToolCallOllama {
            id: Some("call-1".into()),
            extra_content: None,
            function: ToolCallFunction {
                name: "lookup".into(),
                arguments: serde_json::json!({}),
            },
        }]),
    );
    let messages = [
        assistant,
        ChatMessage::tool("result".into(), Some("call-1".into()), None),
    ];

    let payload = payload("deepseek", "deepseek-v4-flash", &messages, &target, "high")
        .expect("tool continuity payload");

    assert_eq!(payload["messages"][0]["reasoning_content"], "opaque-tool");
}

#[test]
fn deepseek_user_continuation_emits_neither_reasoning_nor_replay_evidence() {
    let target = fixture_target(replay_target(
        RouteId::DeepSeek,
        "deepseek-v4-flash",
        ReasoningModeId::High,
        ContinuationUse::UserContinuation,
    ));
    let replay = target.replay().unwrap();
    let messages = [
        ChatMessage::assistant(
            "answer".into(),
            None,
            Some(envelope(
                replay,
                ContractId::DeepSeekChatV1,
                CompletionState::Complete,
                ContinuationState::ChatReasoning {
                    reasoning_content: "opaque-user".into(),
                },
            )),
            None,
            None,
        ),
        ChatMessage::user("follow up".into()),
    ];

    let cfg = RequestConfig {
        provider_id: "deepseek",
        model: "deepseek-v4-flash",
        messages: &messages,
        tools: &[],
        think: true,
        reasoning_mode: Some("high"),
        max_tokens: None,
        purpose: crate::services::llm::request_purpose::RequestPurpose::ManualChat,
        session_id: None,
        fast_mode: FastModeRequest::Unsupported,
        continuation_target: Some(&target),
    };
    let prepared =
        build_chat_payload_with_evidence(&cfg, &route::resolve("deepseek").unwrap(), None)
            .expect("forbidden user replay remains a regular payload");

    assert!(prepared.payload["messages"][0]
        .get("reasoning_content")
        .is_none());
    assert!(prepared.replayed.is_empty());
}

#[tokio::test]
async fn required_missing_envelope_blocks_before_the_transport_records_a_request() {
    let target = fixture_target(replay_target(
        RouteId::Moonshot,
        "kimi-k2.7-code",
        ReasoningModeId::Auto,
        ContinuationUse::UserContinuation,
    ));
    let messages = [
        ChatMessage::assistant("prior".into(), None, None, None, None),
        ChatMessage::user("continue".into()),
    ];
    let scenario = StreamScenario::start("required-no-network", [ScriptedResponse::Success]).await;
    let cfg = RequestConfig {
        provider_id: "moonshot",
        model: "kimi-k2.7-code",
        messages: &messages,
        tools: &[],
        think: true,
        reasoning_mode: Some("auto"),
        max_tokens: None,
        purpose: crate::services::llm::request_purpose::RequestPurpose::ManualChat,
        session_id: Some("required-no-network"),
        fast_mode: FastModeRequest::Unsupported,
        continuation_target: Some(&target),
    };

    let result = crate::services::llm::stream_http::post_chat_request_with_timeout(
        &cfg,
        Duration::from_millis(10),
    )
    .await;

    assert!(result.is_err());
    assert!(
        scenario.payloads().is_empty(),
        "no first or retry request may leave Beaver"
    );
}

#[test]
fn required_rejects_partial_and_provenance_mismatch_before_serialization() {
    let target = fixture_target(replay_target(
        RouteId::Moonshot,
        "kimi-k2.7-code",
        ReasoningModeId::Auto,
        ContinuationUse::UserContinuation,
    ));
    let replay = target.replay().unwrap();
    let partial = [
        ChatMessage::assistant(
            "prior".into(),
            None,
            Some(envelope(
                replay,
                ContractId::KimiChatV1,
                CompletionState::Partial,
                ContinuationState::ChatReasoning {
                    reasoning_content: "partial".into(),
                },
            )),
            None,
            None,
        ),
        ChatMessage::user("continue".into()),
    ];
    let mut compacted = partial.clone();
    compacted[0]
        .continuation
        .as_mut()
        .expect("assistant envelope")
        .completion = CompletionState::Compacted;
    let mismatched = replay_target(
        RouteId::Moonshot,
        "other-kimi-model",
        ReasoningModeId::Auto,
        ContinuationUse::UserContinuation,
    );
    let wrong_model = [
        ChatMessage::assistant(
            "prior".into(),
            None,
            Some(envelope(
                &mismatched,
                ContractId::KimiChatV1,
                CompletionState::Complete,
                ContinuationState::ChatReasoning {
                    reasoning_content: "wrong-model".into(),
                },
            )),
            None,
            None,
        ),
        ChatMessage::user("continue".into()),
    ];
    let mut other_scope = replay.clone();
    other_scope.credential_scope = CredentialScope::authenticated("other-scope").unwrap();
    let wrong_scope = [
        ChatMessage::assistant(
            "prior".into(),
            None,
            Some(envelope(
                &other_scope,
                ContractId::KimiChatV1,
                CompletionState::Complete,
                ContinuationState::ChatReasoning {
                    reasoning_content: "wrong-scope".into(),
                },
            )),
            None,
            None,
        ),
        ChatMessage::user("continue".into()),
    ];
    let mut other_mode = replay.clone();
    other_mode.reasoning_mode = ReasoningModeId::High;
    let wrong_mode = [
        ChatMessage::assistant(
            "prior".into(),
            None,
            Some(envelope(
                &other_mode,
                ContractId::KimiChatV1,
                CompletionState::Complete,
                ContinuationState::ChatReasoning {
                    reasoning_content: "wrong-mode".into(),
                },
            )),
            None,
            None,
        ),
        ChatMessage::user("continue".into()),
    ];

    assert!(payload("moonshot", "kimi-k2.7-code", &partial, &target, "auto").is_err());
    assert!(payload("moonshot", "kimi-k2.7-code", &compacted, &target, "auto").is_err());
    assert!(payload("moonshot", "kimi-k2.7-code", &wrong_model, &target, "auto").is_err());
    assert!(payload("moonshot", "kimi-k2.7-code", &wrong_scope, &target, "auto").is_err());
    assert!(payload("moonshot", "kimi-k2.7-code", &wrong_mode, &target, "auto").is_err());
}

#[test]
fn optional_present_invalid_envelope_blocks_before_serialization() {
    let target = fixture_target(replay_target(
        RouteId::Cerebras,
        "zai-glm-4.7",
        ReasoningModeId::Auto,
        ContinuationUse::UserContinuation,
    ));
    let replay = target.replay().unwrap();
    let messages = [
        ChatMessage::assistant(
            "prior".into(),
            None,
            Some(envelope(
                replay,
                ContractId::CerebrasChatV1,
                CompletionState::Partial,
                ContinuationState::CerebrasReasoning {
                    reasoning: "partial".into(),
                },
            )),
            None,
            None,
        ),
        ChatMessage::user("continue".into()),
    ];

    assert!(payload("cerebras", "zai-glm-4.7", &messages, &target, "auto").is_err());
}

#[test]
fn replay_target_without_a_registered_policy_blocks_chat_payload() {
    let target = ContinuationTarget::Replay(replay_target(
        RouteId::Moonshot,
        "unregistered-kimi-model",
        ReasoningModeId::Auto,
        ContinuationUse::UserContinuation,
    ));
    let messages = [
        ChatMessage::assistant("prior".into(), None, None, None, None),
        ChatMessage::user("continue".into()),
    ];

    assert!(payload(
        "moonshot",
        "unregistered-kimi-model",
        &messages,
        &target,
        "auto"
    )
    .is_err());
}

#[test]
fn kimi_keeps_an_empty_native_reasoning_field_and_legacy_text_is_never_a_fallback() {
    let target = fixture_target(replay_target(
        RouteId::Moonshot,
        "kimi-k2.7-code",
        ReasoningModeId::Auto,
        ContinuationUse::UserContinuation,
    ));
    let replay = target.replay().unwrap();
    let mut legacy = ChatMessage::assistant("legacy".into(), None, None, None, None);
    legacy.tool_loop_reasoning = Some("flat-legacy-text".into());
    let messages = [
        ChatMessage::assistant(
            "native".into(),
            None,
            Some(envelope(
                replay,
                ContractId::KimiChatV1,
                CompletionState::Complete,
                ContinuationState::ChatReasoning {
                    reasoning_content: String::new(),
                },
            )),
            None,
            None,
        ),
        ChatMessage::user("continue".into()),
    ];

    let payload = payload("moonshot", "kimi-k2.7-code", &messages, &target, "auto")
        .expect("empty native continuation remains valid");

    assert_eq!(payload["messages"][0]["reasoning_content"], "");
    assert!(
        crate::services::llm::stream_convert::message_to_openai(&legacy, "moonshot")
            .get("reasoning_content")
            .is_none()
    );
}

#[test]
fn zai_and_cerebras_apply_their_distinct_native_contracts() {
    let zai_target = fixture_target(replay_target(
        RouteId::Zai,
        "glm-5.3",
        ReasoningModeId::Max,
        ContinuationUse::UserContinuation,
    ));
    let zai_replay = zai_target.replay().unwrap();
    let zai_messages = [
        ChatMessage::assistant(
            "answer".into(),
            None,
            Some(envelope(
                zai_replay,
                ContractId::ZaiChatV1,
                CompletionState::Complete,
                ContinuationState::ChatReasoning {
                    reasoning_content: "zai-opaque".into(),
                },
            )),
            None,
            None,
        ),
        ChatMessage::user("continue".into()),
    ];
    let zai =
        payload("zai", "glm-5.3", &zai_messages, &zai_target, "max").expect("zai fixture payload");

    let cerebras_target = fixture_target(replay_target(
        RouteId::Cerebras,
        "zai-glm-4.7",
        ReasoningModeId::Auto,
        ContinuationUse::UserContinuation,
    ));
    let cerebras_replay = cerebras_target.replay().unwrap();
    let cerebras_messages = [
        ChatMessage::assistant(
            "answer".into(),
            None,
            Some(envelope(
                cerebras_replay,
                ContractId::CerebrasChatV1,
                CompletionState::Complete,
                ContinuationState::CerebrasReasoning {
                    reasoning: "cerebras-opaque".into(),
                },
            )),
            None,
            None,
        ),
        ChatMessage::user("continue".into()),
    ];
    let cerebras = payload(
        "cerebras",
        "zai-glm-4.7",
        &cerebras_messages,
        &cerebras_target,
        "auto",
    )
    .expect("cerebras fixture payload");

    let clear = serde_json::json!({"type": "enabled", "clear_thinking": false});
    assert_eq!(zai["messages"][0]["reasoning_content"], "zai-opaque");
    assert_eq!(zai["thinking"], clear);
    assert_eq!(
        cerebras["messages"][0]["content"],
        "<think>cerebras-opaque</think>answer"
    );
    assert!(cerebras["messages"][0].get("reasoning").is_none());
    assert!(cerebras.get("thinking").is_none());
}

#[test]
fn first_required_request_without_an_assistant_remains_allowed() {
    let target = fixture_target(replay_target(
        RouteId::Moonshot,
        "kimi-k2.7-code",
        ReasoningModeId::Auto,
        ContinuationUse::UserContinuation,
    ));
    let messages = [ChatMessage::user("first request".into())];

    let payload = payload("moonshot", "kimi-k2.7-code", &messages, &target, "auto")
        .expect("a required contract has no prior assistant to replay yet");

    assert_eq!(payload["messages"][0]["content"], "first request");
}

#[test]
fn fixture_candidate_bypasses_only_activation_and_never_the_provenance_contract() {
    let replay = replay_target(
        RouteId::Cerebras,
        "zai-glm-4.7",
        ReasoningModeId::Auto,
        ContinuationUse::UserContinuation,
    );
    let envelope = envelope(
        &replay,
        ContractId::CerebrasChatV1,
        CompletionState::Complete,
        ContinuationState::CerebrasReasoning {
            reasoning: "opaque".into(),
        },
    );
    let messages = [
        ChatMessage::assistant("prior".into(), None, Some(envelope), None, None),
        ChatMessage::user("continue".into()),
    ];
    let production = ContinuationTarget::Replay(replay.clone());
    let fixture = ContinuationTarget::FixtureCandidate(replay);

    assert!(payload("cerebras", "zai-glm-4.7", &messages, &production, "auto").is_err());
    assert!(payload("cerebras", "zai-glm-4.7", &messages, &fixture, "auto").is_ok());
}

#[test]
fn groq_reasoning_is_forbidden() {
    let target = ContinuationTarget::Forbidden(NonReplayTarget {
        route_id: RouteId::Groq,
        model_id: "openai/gpt-oss-120b".into(),
        reasoning_mode: ReasoningModeId::High,
    });
    let mut assistant = ChatMessage::assistant("prior".into(), None, None, None, None);
    assistant.tool_loop_reasoning = Some("must-not-forward".into());
    let messages = [assistant, ChatMessage::user("continue".into())];

    let payload = payload("groq", "openai/gpt-oss-120b", &messages, &target, "high")
        .expect("Groq remains a regular request without a replay adapter");

    assert!(payload["messages"][0].get("reasoning_content").is_none());
    assert!(payload["messages"][0].get("reasoning").is_none());
}
