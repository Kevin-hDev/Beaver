use super::*;

#[test]
fn trimming_drops_reasoning_and_respects_the_exact_budget() {
    let message = ChatMessage {
        role: "assistant".into(),
        content: "visible".repeat(10_000),
        reasoning_content: Some("hidden".repeat(20_000)),
        ..Default::default()
    };

    let trimmed = trim_message(&message, 100);

    assert!(trimmed.reasoning_content.is_none());
    assert!(crate::services::token_counting::estimate_chat_message_tokens(&trimmed) <= 100);
    assert!(trimmed.content.contains("message truncated"));
}

#[test]
fn wide_characters_and_a_tiny_budget_cannot_overflow() {
    let message = ChatMessage {
        role: "user".into(),
        content: "你🙂".repeat(1_000),
        ..Default::default()
    };

    for budget in [0, 1, 10, 100] {
        let trimmed = trim_message(&message, budget);
        assert!(
            crate::services::token_counting::estimate_chat_message_tokens(&trimmed) <= budget
        );
    }
}
