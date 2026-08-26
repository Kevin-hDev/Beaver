use super::*;

#[test]
fn trimming_drops_reasoning_and_respects_the_exact_budget() {
    let message = ChatMessage::assistant(
        "visible".repeat(10_000),
        Some("hidden".repeat(20_000)),
        None,
        Some("hidden".repeat(20_000)),
        None,
    );

    let trimmed = trim_message(&message, 100);

    assert!(trimmed.display_thinking.is_none());
    assert!(trimmed.continuation.is_none());
    assert!(trimmed.legacy_tool_loop_reasoning.is_none());
    assert!(crate::services::token_counting::estimate_chat_message_tokens(&trimmed) <= 100);
    assert!(trimmed.content.contains("message truncated"));
}

#[test]
fn wide_characters_and_a_tiny_budget_cannot_overflow() {
    let message = ChatMessage::user("你🙂".repeat(1_000));

    for budget in [0, 1, 10, 100] {
        let trimmed = trim_message(&message, budget);
        assert!(
            crate::services::token_counting::estimate_chat_message_tokens(&trimmed) <= budget
        );
    }
}

#[test]
fn oversized_continuation_fails_closed_instead_of_dropping_opaque_state() {
    use crate::services::reasoning_continuity::contract::{
        ContractId, CredentialScope, ReasoningModeId, RouteId,
    };
    use crate::services::reasoning_continuity::envelope::{
        CompletionState, ContinuationState, ReasoningEnvelope, ReasoningSource,
    };

    let envelope = ReasoningEnvelope::new(
        ContractId::OllamaNativeV1,
        ReasoningSource {
            route_id: RouteId::Ollama,
            model_id: "qwen3.5:4b".into(),
            credential_scope: CredentialScope::local_uncredentialed(),
            reasoning_mode: ReasoningModeId::Auto,
        },
        CompletionState::Complete,
        ContinuationState::OllamaNative {
            thinking: "opaque".repeat(8_000),
        },
        Vec::new(),
    );
    let mut messages = vec![ChatMessage::assistant(
        "visible".repeat(8_000),
        None,
        Some(envelope),
        None,
        None,
    )];

    let error = super::super::context_budget::prepare_for_request(
        &mut messages,
        8_000,
        &[],
        "ollama",
    )
    .expect_err("opaque continuation cannot be partially trimmed");

    assert_eq!(error, "context_capacity_exceeded");
    assert!(messages[0].continuation.is_some());
}
