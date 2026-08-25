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
