use super::*;
use crate::services::agent_local::types_ollama::{ToolCallFunction, ToolCallOllama};
use crate::services::reasoning_continuity::contract::{
    ContinuationTarget, ContinuationUse, ContractId, CredentialScope, ReasoningModeId,
    ReplayTarget, RouteId,
};
use crate::services::reasoning_continuity::envelope::{
    CompletionState, ContinuationState, ReasoningEnvelope, ReasoningSource,
};
use serde_json::json;

fn assistant_with_reasoning() -> ChatMessage {
    ChatMessage::assistant(
        "".to_string(),
        Some("réflexion".to_string()),
        None,
        Some("réflexion".to_string()),
        Some(vec![ToolCallOllama {
            id: Some("call_1".to_string()),
            extra_content: None,
            function: ToolCallFunction {
                name: "grep".to_string(),
                arguments: json!({"pattern": "x"}),
            },
        }]),
    )
}

fn fixture(
    route_id: RouteId,
    model_id: &str,
    reasoning_mode: ReasoningModeId,
    contract_id: ContractId,
    state: ContinuationState,
) -> (ContinuationTarget, ChatMessage) {
    let replay = ReplayTarget {
        route_id,
        model_id: model_id.into(),
        credential_scope: CredentialScope::authenticated("fixture-scope").unwrap(),
        reasoning_mode,
        continuation_use: ContinuationUse::UserContinuation,
    };
    let assistant = ChatMessage::assistant(
        "answer".into(),
        None,
        Some(ReasoningEnvelope::new(
            contract_id,
            ReasoningSource::from_target(&replay),
            CompletionState::Complete,
            state,
            Vec::new(),
        )),
        None,
        None,
    );
    (ContinuationTarget::FixtureCandidate(replay), assistant)
}

#[test]
fn responses_payload_without_native_envelope_drops_legacy_reasoning() {
    let stats = responses_payload_stats(&[assistant_with_reasoning()], None);
    assert_eq!(stats.reasoning_fields, 0);
    assert_eq!(stats.reasoning_chars, 0);
    assert_eq!(stats.assistant_items, 0);
    assert_eq!(stats.tool_calls, 1);
}

#[test]
fn chat_payload_without_approved_target_drops_legacy_reasoning() {
    let stats = chat_payload_stats("zai", &[assistant_with_reasoning()], None);
    assert_eq!(stats.reasoning_fields, 0);
    assert_eq!(stats.reasoning_chars, 0);
    assert_eq!(stats.assistant_content_nulls, 1);
    assert_eq!(stats.tool_calls, 1);
}

#[test]
fn responses_payload_counts_replayed_native_item() {
    let (target, assistant) = fixture(
        RouteId::CodexOauth,
        "gpt-5.6-luna",
        ReasoningModeId::Medium,
        ContractId::CodexResponsesV1,
        ContinuationState::ResponsesLocal {
            items: vec![json!({"type":"reasoning","encrypted_content":"opaque"})],
        },
    );
    let stats = responses_payload_stats(
        &[assistant, ChatMessage::user("next".into())],
        Some(&target),
    );
    assert_eq!(stats.reasoning_fields, 1);
    assert!(stats.reasoning_chars > 0);
}

#[test]
fn chat_payload_counts_replayed_native_field() {
    let (target, assistant) = fixture(
        RouteId::Zai,
        "glm-4.5-flash",
        ReasoningModeId::Auto,
        ContractId::ZaiChatV1,
        ContinuationState::ChatReasoning {
            reasoning_content: "opaque".into(),
        },
    );
    let stats = chat_payload_stats(
        "zai",
        &[assistant, ChatMessage::user("next".into())],
        Some(&target),
    );
    assert_eq!(stats.reasoning_fields, 1);
    assert_eq!(stats.reasoning_chars, 6);
}

#[test]
fn openrouter_diagnostics_builds_a_bare_provider_object_without_panicking() {
    let (target, assistant) = fixture(
        RouteId::OpenRouter,
        "moonshotai/kimi-k2.5",
        ReasoningModeId::Medium,
        ContractId::OpenRouterDetailsV1,
        ContinuationState::OpenRouterDetails {
            details: vec![json!({"type":"reasoning.encrypted","data":"opaque"})],
        },
    );
    let stats = chat_payload_stats(
        "openrouter",
        &[assistant, ChatMessage::user("next".into())],
        Some(&target),
    );

    assert_eq!(stats.reasoning_fields, 1);
    assert!(stats.reasoning_chars > 0);
}

#[test]
fn anthropic_diagnostics_uses_the_native_payload_kind_without_exposing_blocks() {
    let (target, assistant) = fixture(
        RouteId::Anthropic,
        "claude-haiku-4-5-20251001",
        ReasoningModeId::Low,
        ContractId::AnthropicMessagesV1,
        ContinuationState::AnthropicBlocks {
            blocks: vec![
                json!({"type":"thinking","thinking":"opaque","signature":"AAE+/=="}),
                json!({"type":"text","text":"answer"}),
            ],
        },
    );
    let stats = anthropic_payload_stats(
        &[assistant, ChatMessage::user("next".into())],
        Some(&target),
    );

    assert_eq!(
        crate::services::llm::route_profile::diagnostic_payload_kind("anthropic"),
        Some("anthropic_messages")
    );
    assert_eq!(stats.reasoning_fields, 1);
    assert!(stats.reasoning_chars > 0);
    assert_eq!(stats.assistant_content_chars, 6);
}

#[test]
fn qwen_diagnostics_counts_replay_without_exposing_reasoning_text() {
    let (target, assistant) = fixture(
        RouteId::Qwen,
        "qwen3.8-flash",
        ReasoningModeId::Xhigh,
        ContractId::QwenChatV1,
        ContinuationState::ChatReasoning {
            reasoning_content: "opaque-qwen-secret".into(),
        },
    );
    let stats = chat_payload_stats(
        "qwen",
        &[assistant, ChatMessage::user("next".into())],
        Some(&target),
    );

    assert_eq!(stats.reasoning_fields, 1);
    assert_eq!(stats.reasoning_chars, 18);
    let diagnostic = format!("{stats:?}");
    assert!(!diagnostic.contains("opaque-qwen-secret"));
}
