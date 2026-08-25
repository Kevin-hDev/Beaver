use super::contract::{ContractId, CredentialScope, ReasoningModeId, RouteId};
use super::eligibility::{decide, BlockReason, ReplayDecision};
use super::envelope::{CompletionState, ContinuationState, ReasoningEnvelope, ReasoningSource};
use super::limits::{
    checked_envelope_bytes, checked_session_continuity_bytes, checked_tool_calls,
    validate_session_continuity_bytes, CaptureBudget, LimitError, MAX_ENVELOPE_BYTES,
    MAX_NATIVE_ITEMS, MAX_SESSION_CONTINUITY_BYTES, MAX_TOOL_CALLS,
};
use super::tool_links::ToolLink;
use serde_json::{json, Value};

fn scope(value: &str) -> CredentialScope {
    CredentialScope::authenticated(value).expect("valid fixture scope")
}

fn source() -> ReasoningSource {
    ReasoningSource {
        route_id: RouteId::OpenRouter,
        model_id: "moonshotai/kimi-k2.5".into(),
        credential_scope: scope("fixture-scope"),
        reasoning_mode: ReasoningModeId::High,
    }
}

fn envelope(completion: CompletionState) -> ReasoningEnvelope {
    ReasoningEnvelope::new(
        ContractId::OpenRouterDetailsV1,
        source(),
        completion,
        ContinuationState::OpenRouterDetails {
            details: vec![json!({"type": "summary", "index": 0})],
        },
        Vec::new(),
    )
}

fn target() -> super::contract::ReplayTarget {
    super::contract::ReplayTarget {
        route_id: RouteId::OpenRouter,
        model_id: "moonshotai/kimi-k2.5".into(),
        credential_scope: scope("fixture-scope"),
        reasoning_mode: ReasoningModeId::High,
    }
}

#[test]
fn r02_all_native_variants_preserve_item_order_through_serde() {
    let ordered = vec![
        json!({"index": 0, "type": "first"}),
        json!({"index": 1, "type": "second"}),
        json!({"index": 2, "type": "third"}),
    ];
    let states = [
        ContinuationState::OllamaNative {
            thinking: "one\ntwo".into(),
        },
        ContinuationState::ChatReasoning {
            reasoning_content: "one\ntwo".into(),
        },
        ContinuationState::CerebrasReasoning {
            reasoning: "one\ntwo".into(),
        },
        ContinuationState::GeminiParts {
            parts: ordered.clone(),
        },
        ContinuationState::MistralChunks {
            chunks: ordered.clone(),
        },
        ContinuationState::OpenRouterDetails {
            details: ordered.clone(),
        },
        ContinuationState::ResponsesLocal {
            items: ordered.clone(),
        },
        ContinuationState::RemoteContinuation {
            response_id: "response-ordered".into(),
        },
    ];

    for state in states {
        let bytes = serde_json::to_vec(&state).unwrap();
        let restored: ContinuationState = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(restored, state);
    }
}

#[test]
fn r03_opaque_and_encoded_strings_are_byte_identical_after_round_trip() {
    let opaque = "AAECAwQF+/==.signed\u{0000}tail";
    let state = ContinuationState::GeminiParts {
        parts: vec![json!({
            "thoughtSignature": opaque,
            "inlineData": {"data": opaque}
        })],
    };

    let restored: ContinuationState =
        serde_json::from_slice(&serde_json::to_vec(&state).unwrap()).unwrap();
    assert_eq!(restored, state);
    let ContinuationState::GeminiParts { parts } = restored else {
        panic!("wrong variant")
    };
    assert_eq!(parts[0]["thoughtSignature"].as_str(), Some(opaque));
    assert_eq!(parts[0]["inlineData"]["data"].as_str(), Some(opaque));
}

#[test]
fn r05_partial_and_compacted_envelopes_are_never_replayed() {
    assert_eq!(
        decide(&envelope(CompletionState::Partial), &target()),
        ReplayDecision::Blocked(BlockReason::PartialState)
    );
    assert_eq!(
        decide(&envelope(CompletionState::Compacted), &target()),
        ReplayDecision::Blocked(BlockReason::CompactedState)
    );
}

#[test]
fn r06_every_provenance_difference_blocks_replay() {
    let complete = envelope(CompletionState::Complete);
    let mut cases = Vec::new();

    let mut other_route = target();
    other_route.route_id = RouteId::Moonshot;
    other_route.model_id = "kimi-k2.7-code".into();
    cases.push(other_route);

    let mut other_model = target();
    other_model.model_id = "stealth/ox-alpha".into();
    cases.push(other_model);

    let mut other_scope = target();
    other_scope.credential_scope = scope("other-scope");
    cases.push(other_scope);

    let mut other_mode = target();
    other_mode.reasoning_mode = ReasoningModeId::Low;
    cases.push(other_mode);

    for different in cases {
        assert_eq!(
            decide(&complete, &different),
            ReplayDecision::Blocked(BlockReason::ProvenanceMismatch)
        );
    }
}

#[test]
fn r07_incremental_item_limit_rejects_the_whole_capture() {
    let mut budget = CaptureBudget::new();
    for index in 0..MAX_NATIVE_ITEMS {
        budget.observe_item(&json!({"index": index})).unwrap();
    }
    let before = budget;

    assert_eq!(
        budget.observe_item(&json!({"index": MAX_NATIVE_ITEMS})),
        Err(LimitError::NativeItems)
    );
    assert_eq!(budget, before);
}

#[test]
fn r07_envelope_tool_and_session_limits_reject_without_truncation() {
    let oversized = ContinuationState::ChatReasoning {
        reasoning_content: "x".repeat(MAX_ENVELOPE_BYTES + 1),
    };
    let oversized = ReasoningEnvelope::new(
        ContractId::DeepSeekChatV1,
        ReasoningSource {
            route_id: RouteId::DeepSeek,
            model_id: "deepseek-v4-flash".into(),
            credential_scope: scope("fixture-scope"),
            reasoning_mode: ReasoningModeId::High,
        },
        CompletionState::Complete,
        oversized,
        Vec::new(),
    );
    assert_eq!(oversized.validate(), Err(LimitError::EnvelopeBytes));

    let links = (0..65)
        .map(|index| ToolLink {
            provider_call_id: format!("call-{index}"),
            tool_name: "fixture".into(),
        })
        .collect();
    let too_many_links = ReasoningEnvelope::new(
        ContractId::OpenRouterDetailsV1,
        source(),
        CompletionState::Complete,
        ContinuationState::OpenRouterDetails {
            details: Vec::new(),
        },
        links,
    );
    assert_eq!(too_many_links.validate(), Err(LimitError::ToolCalls));

    assert_eq!(
        validate_session_continuity_bytes(MAX_SESSION_CONTINUITY_BYTES + 1),
        Err(LimitError::SessionBytes)
    );
}

#[test]
fn r07_json_depth_is_bounded_before_native_items_are_retained() {
    let mut value = Value::Null;
    for _ in 0..=super::limits::MAX_JSON_DEPTH {
        value = json!([value]);
    }
    let mut budget = CaptureBudget::new();
    assert_eq!(budget.observe_item(&value), Err(LimitError::JsonDepth));
    assert_eq!(budget.item_count(), 0);
    assert_eq!(budget.serialized_bytes(), 0);
}

#[test]
fn r07_incremental_byte_additions_are_checked_before_mutation() {
    let mut budget = CaptureBudget::new();
    budget.observe_serialized_bytes(MAX_ENVELOPE_BYTES).unwrap();
    let before = budget;

    assert_eq!(
        budget.observe_serialized_bytes(1),
        Err(LimitError::EnvelopeBytes)
    );
    assert_eq!(budget, before);
    assert_eq!(
        checked_envelope_bytes(1, usize::MAX),
        Err(LimitError::ArithmeticOverflow)
    );
    assert_eq!(
        checked_tool_calls(MAX_TOOL_CALLS, 1),
        Err(LimitError::ToolCalls)
    );
    assert_eq!(
        checked_tool_calls(1, usize::MAX),
        Err(LimitError::ArithmeticOverflow)
    );
    assert_eq!(
        checked_session_continuity_bytes(MAX_SESSION_CONTINUITY_BYTES, 1),
        Err(LimitError::SessionBytes)
    );
    assert_eq!(
        checked_session_continuity_bytes(1, usize::MAX),
        Err(LimitError::ArithmeticOverflow)
    );
}
