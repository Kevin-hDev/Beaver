use super::*;
use crate::services::agent_local::types_ollama::{ToolCallFunction, ToolCallOllama};
use crate::services::reasoning_continuity::contract::{
    ContinuationUse, ContractId, CredentialScope, ReasoningModeId, ReplayTarget, RouteId,
};
use crate::services::reasoning_continuity::envelope::{
    CompletionState, ContinuationState, ReasoningEnvelope, ReasoningSource,
};

fn target(model_id: &str) -> ReplayTarget {
    ReplayTarget {
        route_id: RouteId::Ollama,
        model_id: model_id.into(),
        credential_scope: CredentialScope::local_uncredentialed(),
        reasoning_mode: ReasoningModeId::Auto,
        continuation_use: ContinuationUse::UserContinuation,
    }
}

fn message(target: &ReplayTarget, thinking: &str) -> ChatMessage {
    ChatMessage::assistant(
        "answer".into(),
        None,
        Some(ReasoningEnvelope::new(
            ContractId::OllamaNativeV1,
            ReasoningSource::from_target(target),
            CompletionState::Complete,
            ContinuationState::OllamaNative { thinking: thinking.into() },
            Vec::new(),
        )),
        None,
        None,
    )
}

fn request() -> ChatRequest {
    ChatRequest {
        model: "qwen3.5:4b".into(),
        messages: Vec::new(),
        stream: true,
        tools: None,
        options: None,
        keep_alive: None,
        think: None,
        capture_reasoning: false,
        live_replay_target: None,
        fixture_candidate: None,
    }
}

#[test]
fn native_payload_uses_ollama_thinking_and_strips_local_tool_ids() {
    let mut message = ChatMessage::assistant(
        String::new(),
        Some("raisonnement".into()),
        None,
        Some("raisonnement".into()),
        Some(vec![ToolCallOllama {
            id: Some("0f7a0a1a-0000-4000-8000-000000000001".into()),
            extra_content: Some(json!({"provider": "api"})),
            function: ToolCallFunction { name: "search".into(), arguments: json!({"query": "test"}) },
        }]),
    );
    message.tool_call_id = Some("0f7a0a1a-0000-4000-8000-000000000002".into());

    let value = messages_value(&[message]);
    let serialized = value.to_string();
    assert_eq!(value[0]["thinking"], "raisonnement");
    assert!(!serialized.contains("reasoning_content"));
    assert!(!serialized.contains("tool_call_id"));
    assert!(!serialized.contains("extra_content"));
    assert!(!serialized.contains("0f7a0a1a-0000-4000-8000-000000000001"));
    assert!(!serialized.contains("0f7a0a1a-0000-4000-8000-000000000002"));
}

#[test]
fn chat_payload_disables_ollama_truncation() {
    let value = chat_request(&request(), &[]).unwrap();
    assert_eq!(value["truncate"], false);
    assert!(value.get("think").is_none());
}

#[test]
fn only_live_validated_qwen_replays_in_production() {
    let qwen = target("qwen3.5:4b");
    let messages = [message(&qwen, "opaque historic")];
    let mut live = request();
    live.live_replay_target = Some(qwen.clone());
    assert_eq!(chat_request(&live, &messages).unwrap()["messages"][0]["thinking"], "opaque historic");

    live.live_replay_target = Some(target("gemma4:e2b-it-q4_K_M"));
    assert_eq!(
        chat_request(&live, &messages),
        Err(crate::services::llm::reasoning_wire::replay::ReplayApplyError::Blocked)
    );
}

#[cfg(debug_assertions)]
#[test]
fn fixture_candidate_replays_multiple_native_messages_while_normal_stays_closed() {
    let qwen = target("qwen3.5:4b");
    let messages = [message(&qwen, "opaque historic"), message(&qwen, "opaque later")];
    let mut candidate = request();
    let normal = chat_request(&candidate, &messages).unwrap();
    assert!(normal["messages"][0].get("thinking").is_none());
    assert!(normal["messages"][1].get("thinking").is_none());
    candidate.fixture_candidate = Some(qwen);
    let replay = chat_request(&candidate, &messages).unwrap();
    assert_eq!(replay["messages"][0]["thinking"], "opaque historic");
    assert_eq!(replay["messages"][1]["thinking"], "opaque later");
}
