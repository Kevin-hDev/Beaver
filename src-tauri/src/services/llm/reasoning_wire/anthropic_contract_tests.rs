use super::replay::{apply_anthropic_continuity, approval_for_target};
use crate::services::agent_local::types_ollama::{ChatMessage, ToolCallFunction, ToolCallOllama};
use crate::services::reasoning_continuity::contract::{
    ContinuationTarget, ContinuationUse, ContractId, CredentialScope, ReasoningModeId,
    ReplayTarget, RouteId,
};
use crate::services::reasoning_continuity::envelope::{
    CompletionState, ContinuationState, ReasoningEnvelope, ReasoningSource,
};
use crate::services::reasoning_continuity::tool_links::ToolLink;
use serde_json::{json, Value};

fn target() -> ContinuationTarget {
    ContinuationTarget::FixtureCandidate(ReplayTarget {
        route_id: RouteId::Anthropic,
        model_id: "claude-haiku-4-5-20251001".into(),
        credential_scope: CredentialScope::authenticated("fixture-scope").unwrap(),
        reasoning_mode: ReasoningModeId::Low,
        continuation_use: ContinuationUse::ToolContinuation,
    })
}

fn blocks() -> Vec<Value> {
    vec![
        json!({"type":"thinking","thinking":"opaque","signature":"AAE+/=="}),
        json!({"type":"redacted_thinking","data":"redacted+/=="}),
        json!({"type":"tool_use","id":"toolu_1","name":"read_file","input":{"path":"README.md"}}),
    ]
}

fn envelope(blocks: Vec<Value>) -> ReasoningEnvelope {
    let target = target();
    let replay = target.replay().unwrap();
    let tool_links = blocks
        .iter()
        .filter(|block| block["type"] == "tool_use")
        .map(|block| ToolLink {
            provider_call_id: block["id"].as_str().unwrap().to_string(),
            tool_name: "read_file".into(),
        })
        .collect();
    ReasoningEnvelope::new(
        ContractId::AnthropicMessagesV1,
        ReasoningSource::from_target(replay),
        CompletionState::Complete,
        ContinuationState::AnthropicBlocks { blocks },
        tool_links,
    )
}

fn assistant(envelope: ReasoningEnvelope) -> ChatMessage {
    ChatMessage::assistant(
        "reconstructed".into(),
        None,
        Some(envelope),
        None,
        Some(vec![ToolCallOllama {
            id: Some("toolu_1".into()),
            extra_content: None,
            function: ToolCallFunction {
                name: "read_file".into(),
                arguments: json!({"path":"README.md"}),
            },
        }]),
    )
}

#[test]
fn anthropic_blocks_keep_order_and_signatures_byte_exact() {
    let state = ContinuationState::AnthropicBlocks { blocks: blocks() };
    let bytes = serde_json::to_vec(&state).unwrap();
    let restored: ContinuationState = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(restored, state);
    assert_eq!(serde_json::to_vec(&restored).unwrap(), bytes);
}

#[test]
fn replay_replaces_reconstructed_assistant_content_then_keeps_tool_result() {
    let envelope = envelope(blocks());
    let messages = vec![
        assistant(envelope.clone()),
        ChatMessage::tool("ok".into(), Some("toolu_1".into()), None),
    ];
    let mut payload = vec![
        json!({"role":"assistant","content":[{"type":"text","text":"reconstructed"}]}),
        json!({"role":"user","content":[{"type":"tool_result","tool_use_id":"toolu_1","content":"ok"}]}),
    ];
    let target = target();
    let approval = approval_for_target(&target, &envelope).unwrap();

    apply_anthropic_continuity(&messages, &approval, &mut payload).unwrap();

    assert_eq!(payload[0]["content"], Value::Array(blocks()));
    assert_eq!(payload[1]["content"][0]["tool_use_id"], "toolu_1");
}

#[test]
fn missing_signature_and_divergent_tool_id_fail_closed() {
    let missing_signature = envelope(vec![json!({"type":"thinking","thinking":"opaque"})]);
    assert!(missing_signature.validate().is_err());

    let valid = envelope(blocks());
    let mut message = assistant(valid.clone());
    message.tool_calls.as_mut().unwrap()[0].id = Some("toolu_other".into());
    let target = target();
    let approval = approval_for_target(&target, &valid).unwrap();
    let mut payload = vec![json!({"role":"assistant","content":[]})];
    assert!(apply_anthropic_continuity(&[message], &approval, &mut payload).is_err());
}

#[test]
fn provenance_and_partial_state_fail_closed() {
    let valid = envelope(blocks());
    let mut cases = Vec::new();

    let mut other_key = target().replay().unwrap().clone();
    other_key.credential_scope = CredentialScope::authenticated("other-scope").unwrap();
    cases.push(ContinuationTarget::FixtureCandidate(other_key));

    let mut other_mode = target().replay().unwrap().clone();
    other_mode.reasoning_mode = ReasoningModeId::High;
    cases.push(ContinuationTarget::FixtureCandidate(other_mode));

    let mut other_model = target().replay().unwrap().clone();
    other_model.model_id = "claude-other".into();
    cases.push(ContinuationTarget::FixtureCandidate(other_model));

    for different in cases {
        assert!(approval_for_target(&different, &valid).is_err());
    }

    let mut partial = valid.clone();
    partial.completion = CompletionState::Partial;
    let target = target();
    assert!(approval_for_target(&target, &partial).is_err());
}

#[test]
fn anthropic_native_limits_reject_depth_items_and_bytes_without_truncation() {
    let mut nested = Value::Null;
    for _ in 0..=crate::services::reasoning_continuity::limits::MAX_JSON_DEPTH {
        nested = json!([nested]);
    }
    assert!(
        envelope(vec![json!({"type":"text","text":"ok","extra":nested})])
            .validate()
            .is_err()
    );

    let too_many = (0..=crate::services::reasoning_continuity::limits::MAX_NATIVE_ITEMS)
        .map(|index| json!({"type":"text","text":index.to_string()}))
        .collect();
    assert!(envelope(too_many).validate().is_err());

    let oversized =
        "x".repeat(crate::services::reasoning_continuity::limits::MAX_ENVELOPE_BYTES + 1);
    assert!(envelope(vec![json!({"type":"text","text":oversized})])
        .validate()
        .is_err());
}
