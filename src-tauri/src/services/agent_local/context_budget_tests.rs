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
    let report = prepare_for_request(&mut messages, 0, &[], "ollama").expect("unknown context");
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
    let report =
        prepare_for_request(&mut messages, 20_000, &[], "ollama").expect("budgeted context");
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

    assert!(prepare_for_request(&mut messages, 4_000, &[], "ollama").is_err());
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

    prepare_for_request(&mut messages, 12_000, &[], "ollama").expect("complete report fits");
    assert!(messages
        .iter()
        .any(|message| message.content == report_content));
}

#[test]
fn tool_definitions_reduce_the_message_budget() {
    let mut messages = vec![
        msg("system", "rules"),
        msg("user", &"old".repeat(8_000)),
        msg("assistant", "recent"),
    ];
    let tools = vec![serde_json::json!({
        "type": "function",
        "function": {
            "name": "large_tool",
            "description": "d".repeat(20_000),
            "parameters": {"type": "object", "properties": {}}
        }
    })];

    let report = prepare_for_request(&mut messages, 12_000, &tools, "ollama").unwrap();

    assert!(report.pruned_messages > 0);
    assert!(report.estimated_tokens <= report.max_input_tokens.unwrap());
}

#[test]
fn payload_reduction_changes_an_oversized_request_once() {
    let mut messages = vec![
        msg("system", "rules"),
        msg("user", &"a".repeat(16_000)),
        msg("assistant", &"b".repeat(16_000)),
    ];
    let before = token_estimate::estimate_tokens(&messages);

    let changed =
        reduce_after_payload_too_large(&mut messages, 100_000, &[], "ollama").unwrap();

    assert!(changed);
    assert!(token_estimate::estimate_tokens(&messages) < before);
}

#[test]
fn codex_does_not_prune_reasoning_that_is_not_sent() {
    let mut messages = vec![msg("system", "rules"), msg("user", &"a".repeat(280_000))];
    messages.push(ChatMessage {
        role: "assistant".into(),
        content: "recent answer".into(),
        reasoning_content: Some("r".repeat(80_000)),
        ..Default::default()
    });
    let original_len = messages.len();
    let original_content = messages[1].content.clone();

    let report = prepare_for_request(
        &mut messages,
        100_000,
        &[],
        crate::services::codex_client::PROVIDER_ID,
    )
    .expect("codex context should fit without hidden reasoning");

    assert_eq!(report.pruned_messages, 0);
    assert_eq!(messages.len(), original_len);
    assert_eq!(messages[1].content, original_content);
}

#[test]
fn pruning_keeps_a_tool_call_and_all_results_together() {
    let mut messages = vec![
        msg("system", "rules"),
        msg("user", &"old".repeat(20_000)),
        assistant_with_calls(&["call-1", "call-2"]),
        tool_message("call-1", "grep", "first"),
        tool_message("call-2", "glob", "second"),
        msg("assistant", "recent answer"),
    ];

    prepare_for_request(&mut messages, 12_000, &[], "openai").unwrap();

    let call_index = messages
        .iter()
        .position(|message| message.tool_calls.is_some())
        .expect("complete tool chain retained");
    assert_eq!(messages[call_index + 1].tool_call_id.as_deref(), Some("call-1"));
    assert_eq!(messages[call_index + 2].tool_call_id.as_deref(), Some("call-2"));
}

#[test]
fn oversized_tool_chain_is_omitted_as_a_whole() {
    let mut messages = vec![
        msg("system", "rules"),
        assistant_with_calls(&["call-1"]),
        tool_message("call-1", "grep", &"huge".repeat(20_000)),
        msg("assistant", "recent answer"),
    ];

    prepare_for_request(&mut messages, 8_000, &[], "openai").unwrap();

    assert!(messages.iter().all(|message| message.tool_calls.is_none()));
    assert!(messages.iter().all(|message| message.role != "tool"));
    assert!(messages.iter().any(|message| message.content.contains("recent answer")));
}

fn assistant_with_calls(ids: &[&str]) -> ChatMessage {
    ChatMessage {
        role: "assistant".into(),
        content: String::new(),
        tool_calls: Some(ids.iter().enumerate().map(|(index, id)| {
            super::super::types_ollama::ToolCallOllama {
                id: Some((*id).into()),
                extra_content: None,
                function: super::super::types_ollama::ToolCallFunction {
                    name: if index == 0 { "grep" } else { "glob" }.into(),
                    arguments: serde_json::json!({}),
                },
            }
        }).collect()),
        ..Default::default()
    }
}

fn tool_message(id: &str, name: &str, content: &str) -> ChatMessage {
    ChatMessage {
        role: "tool".into(),
        content: content.into(),
        tool_name: Some(name.into()),
        tool_call_id: Some(id.into()),
        ..Default::default()
    }
}
