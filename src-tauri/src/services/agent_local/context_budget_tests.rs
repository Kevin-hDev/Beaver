use super::*;

fn msg(role: &str, content: &str) -> ChatMessage {
    ChatMessage {
        role: role.to_string(),
        content: content.to_string(),
        ..Default::default()
    }
}

#[test]
fn unknown_context_does_not_prune() {
    let mut messages = vec![msg("user", &"x".repeat(100_000))];
    let report = prepare_for_request(&mut messages, 0).expect("unknown context");
    assert_eq!(report.max_input_tokens, None);
    assert_eq!(messages.len(), 1);
}

#[test]
fn preserves_system_and_recent_tail() {
    let mut messages = vec![
        msg("system", "rules"),
        msg("user", &"a".repeat(80_000)),
        msg("assistant", "recent"),
    ];
    let report = prepare_for_request(&mut messages, 20_000).expect("budgeted context");
    assert!(report.pruned_messages > 0);
    assert_eq!(messages[0].role, "system");
    assert!(messages.last().unwrap().content.contains("recent"));
}

#[test]
fn oversized_subagent_report_fails_closed_instead_of_truncating() {
    let report_content = format!(
        "{}\n{}",
        super::super::subagent_report_context::SUBAGENT_REPORT_CONTEXT_PREFIX,
        "r".repeat(12_000)
    );
    let mut messages = vec![
        msg("system", "rules"),
        msg("assistant", report_content.as_str()),
    ];

    assert!(prepare_for_request(&mut messages, 4_000).is_err());
    assert_eq!(messages[1].content, report_content);
}

#[test]
fn fitting_subagent_report_survives_saturated_context_intact() {
    let report_content = format!(
        "{}\n{}",
        super::super::subagent_report_context::SUBAGENT_REPORT_CONTEXT_PREFIX,
        "r".repeat(4_000)
    );
    let mut messages = vec![
        msg("system", "rules"),
        msg("user", &"old".repeat(30_000)),
        msg("assistant", report_content.as_str()),
    ];

    prepare_for_request(&mut messages, 12_000).expect("complete report fits");
    assert!(messages
        .iter()
        .any(|message| message.content == report_content));
}
