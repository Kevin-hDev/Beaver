use super::build_chat_payload;
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::llm::fast_mode::FastModeRequest;
use crate::services::llm::route;
use crate::services::llm::stream_http::RequestConfig;
use crate::services::reasoning_continuity::contract::{
    ContinuationTarget, ContinuationUse, ContractId, CredentialScope, ReasoningModeId,
    ReplayTarget, RouteId,
};
use crate::services::reasoning_continuity::envelope::{
    CompletionState, ContinuationState, ReasoningEnvelope, ReasoningSource,
};
use serde_json::json;

fn target() -> ContinuationTarget {
    ContinuationTarget::FixtureCandidate(ReplayTarget {
        route_id: RouteId::Google,
        model_id: "gemini-3.7-flash".into(),
        credential_scope: CredentialScope::authenticated("fixture-scope").unwrap(),
        reasoning_mode: ReasoningModeId::Medium,
        continuation_use: ContinuationUse::UserContinuation,
    })
}

fn replay_target(
    route_id: RouteId,
    model_id: &str,
    reasoning_mode: ReasoningModeId,
) -> ReplayTarget {
    ReplayTarget {
        route_id,
        model_id: model_id.into(),
        credential_scope: CredentialScope::authenticated("fixture-scope").unwrap(),
        reasoning_mode,
        continuation_use: ContinuationUse::UserContinuation,
    }
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
        think: true,
        reasoning_mode: Some(mode),
        max_tokens: None,
        purpose: crate::services::llm::request_purpose::RequestPurpose::ManualChat,
        session_id: None,
        fast_mode: FastModeRequest::Unsupported,
        tool_result_previews: None,
        continuation_target: Some(target),
    };
    build_chat_payload(&cfg, &route::resolve(provider_id).unwrap(), None)
}

#[test]
fn gemini_reasoning_fixture_payload_keeps_parts_and_late_signature_in_order() {
    let target = target();
    let replay = target.replay().unwrap().clone();
    let parts = vec![
        json!({"tool_call": {"index": 0, "extra_content": {"google": {"thought_signature": "tool-signature"}}}}),
        json!({"extra_content": {"google": {"thought_signature": "late-signature"}}}),
    ];
    let envelope = ReasoningEnvelope::new(
        ContractId::GeminiCompatV1,
        ReasoningSource::from_target(&replay),
        CompletionState::Complete,
        ContinuationState::GeminiParts {
            parts: parts.clone(),
        },
        Vec::new(),
    );
    let reloaded: ReasoningEnvelope =
        serde_json::from_slice(&serde_json::to_vec(&envelope).expect("persisted Gemini envelope"))
            .expect("reloaded Gemini envelope");
    let messages = [
        ChatMessage::assistant(
            "answer".into(),
            None,
            Some(reloaded),
            None,
            Some(vec![
                crate::services::agent_local::types_ollama::ToolCallOllama {
                    id: Some("call-1".into()),
                    extra_content: None,
                    function: crate::services::agent_local::types_ollama::ToolCallFunction {
                        name: "lookup".into(),
                        arguments: json!({}),
                    },
                },
            ]),
        ),
        ChatMessage::user("continue".into()),
    ];
    let cfg = RequestConfig {
        provider_id: "google",
        model: "gemini-3.7-flash",
        messages: &messages,
        tools: &[],
        think: true,
        reasoning_mode: Some("medium"),
        max_tokens: None,
        purpose: crate::services::llm::request_purpose::RequestPurpose::ManualChat,
        session_id: None,
        fast_mode: FastModeRequest::Unsupported,
        tool_result_previews: None,
        continuation_target: Some(&target),
    };

    let payload = build_chat_payload(&cfg, &route::resolve("google").unwrap(), None)
        .expect("fixture payload");

    assert_eq!(
        payload["messages"][0]["tool_calls"][0]["extra_content"],
        json!({"google": {"thought_signature": "tool-signature"}})
    );
    assert_eq!(
        payload["messages"][0]["extra_content"],
        json!({"google": {"thought_signature": "late-signature"}})
    );
    assert_eq!(payload["messages"][0]["content"], "answer");
}

#[test]
fn mistral_reasoning_fixture_payload_preserves_interleaved_chunks_and_tool_order_after_reload() {
    let replay = replay_target(
        RouteId::Mistral,
        "mistral-small-2603",
        ReasoningModeId::High,
    );
    let target = ContinuationTarget::FixtureCandidate(replay.clone());
    let chunks = vec![
        json!({"type": "think", "text": "first"}),
        json!({"type": "tool", "id": "call-1", "name": "lookup"}),
        json!({"type": "think", "text": "second"}),
    ];
    let envelope = ReasoningEnvelope::new(
        ContractId::MistralChunksV1,
        ReasoningSource::from_target(&replay),
        CompletionState::Complete,
        ContinuationState::MistralChunks {
            chunks: chunks.clone(),
        },
        Vec::new(),
    );
    let reloaded: ReasoningEnvelope =
        serde_json::from_slice(&serde_json::to_vec(&envelope).expect("persisted Mistral envelope"))
            .expect("reloaded Mistral envelope");
    let messages = [
        ChatMessage::assistant("answer".into(), None, Some(reloaded), None, None),
        ChatMessage::user("continue".into()),
    ];

    let payload = payload("mistral", "mistral-small-2603", &messages, &target, "high")
        .expect("Mistral fixture payload");

    assert_eq!(payload["messages"][0]["content"], json!(chunks));
    assert!(payload["messages"][0].get("reasoning_content").is_none());
}

#[test]
fn openrouter_reasoning_fixture_payload_keeps_encrypted_details_in_order_and_disables_fallbacks() {
    let replay = replay_target(
        RouteId::OpenRouter,
        "moonshotai/kimi-k2.5",
        ReasoningModeId::Medium,
    );
    let target = ContinuationTarget::FixtureCandidate(replay.clone());
    let details = vec![
        json!({"type": "reasoning.encrypted", "data": "AAECAwQ="}),
        json!({"type": "reasoning.summary", "text": "opaque summary"}),
        json!({"type": "reasoning.encrypted", "data": "tail=="}),
    ];
    let envelope = ReasoningEnvelope::new(
        ContractId::OpenRouterDetailsV1,
        ReasoningSource::from_target(&replay),
        CompletionState::Complete,
        ContinuationState::OpenRouterDetails {
            details: details.clone(),
        },
        Vec::new(),
    );
    let reloaded: ReasoningEnvelope = serde_json::from_slice(
        &serde_json::to_vec(&envelope).expect("persisted OpenRouter envelope"),
    )
    .expect("reloaded OpenRouter envelope");
    let messages = [
        ChatMessage::assistant("answer".into(), None, Some(reloaded), None, None),
        ChatMessage::user("continue".into()),
    ];

    let payload = payload(
        "openrouter",
        "moonshotai/kimi-k2.5",
        &messages,
        &target,
        "medium",
    )
    .expect("OpenRouter fixture payload");

    assert_eq!(payload["messages"][0]["reasoning_details"], json!(details));
    assert_eq!(payload["provider"]["allow_fallbacks"], false);
    assert!(payload["messages"][0].get("reasoning_content").is_none());
}
