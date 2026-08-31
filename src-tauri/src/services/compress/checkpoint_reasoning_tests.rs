use super::checkpoint_messages_tests::{limits, message};
use super::checkpoint_selection::select;
use crate::services::reasoning_continuity::contract::{
    ContinuationUse, ContractId, CredentialScope, ReasoningModeId, ReplayTarget, RouteId,
};
use crate::services::reasoning_continuity::eligibility::{decide, BlockReason, ReplayDecision};
use crate::services::reasoning_continuity::envelope::{
    CompletionState, ContinuationState, ReasoningEnvelope, ReasoningSource,
};

fn envelope(reasoning: String) -> ReasoningEnvelope {
    ReasoningEnvelope::new(
        ContractId::OpenRouterDetailsV1,
        ReasoningSource {
            route_id: RouteId::OpenRouter,
            model_id: "model-a".into(),
            credential_scope: CredentialScope::authenticated("scope-a").unwrap(),
            reasoning_mode: ReasoningModeId::Medium,
        },
        CompletionState::Complete,
        ContinuationState::OpenRouterDetails {
            details: vec![serde_json::json!({"reasoning": reasoning})],
        },
        Vec::new(),
    )
}

#[test]
fn refuses_an_opaque_envelope_above_the_existing_eight_mib_limit() {
    let mut assistant = message("old", "assistant", "answer");
    assistant.continuation = Some(envelope("x".repeat(9 * 1024 * 1024)));
    let source = vec![message("old", "user", "q"), assistant];
    assert_eq!(
        select(&source, limits(20_000, 20_000)).unwrap_err(),
        "compression_checkpoint_invalid_reasoning"
    );
}

#[test]
fn valid_reasoning_is_indivisible_and_retained_byte_for_byte_only_if_it_fits() {
    let mut assistant = message("old", "assistant", "answer");
    assistant.continuation = Some(envelope("opaque".repeat(20_000)));
    let original = serde_json::to_vec(&assistant).unwrap();
    let source = vec![
        message("old", "user", "q"),
        assistant,
        message("active", "user", "now"),
    ];
    let omitted = select(&source, limits(5_000, 1_000)).unwrap();
    assert!(!omitted
        .messages
        .iter()
        .any(|item| item.message().id == source[1].id));

    let retained = select(&source, limits(5_000, 50_000)).unwrap();
    let restored = retained
        .messages
        .iter()
        .find(|item| item.message().id == source[1].id)
        .unwrap();
    assert_eq!(serde_json::to_vec(restored.message()).unwrap(), original);
}

#[test]
fn provider_change_and_second_selection_never_retarget_reasoning() {
    let mut assistant = message("old", "assistant", "answer");
    assistant.continuation = Some(envelope("opaque".into()));
    let source = vec![message("old", "user", "q"), assistant];
    let first = select(&source, limits(5_000, 5_000)).unwrap();
    let retained = first
        .messages
        .iter()
        .map(|item| item.message().clone())
        .collect::<Vec<_>>();
    let second = select(&retained, limits(5_000, 5_000)).unwrap();
    let kept = second
        .messages
        .iter()
        .find_map(|item| item.message().continuation.as_ref())
        .unwrap();
    let other_target = ReplayTarget {
        route_id: RouteId::OpenRouter,
        model_id: "model-b".into(),
        credential_scope: CredentialScope::authenticated("scope-a").unwrap(),
        reasoning_mode: ReasoningModeId::Medium,
        continuation_use: ContinuationUse::UserContinuation,
    };
    assert!(matches!(
        decide(kept, &other_target),
        ReplayDecision::Blocked(BlockReason::ProvenanceMismatch | BlockReason::UnknownTarget)
    ));
    assert_eq!(kept.source.model_id, "model-a");
}
