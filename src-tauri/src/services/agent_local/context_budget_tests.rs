use super::*;

fn msg(role: &str, content: &str) -> ChatMessage {
    match role {
"system" => ChatMessage::system(content.to_string()),
"user" => ChatMessage::user(content.to_string()),
"assistant" => ChatMessage::assistant(content.to_string(), None, None),
"tool" => ChatMessage::tool(content.to_string(), None, None),
other => panic!("unsupported chat role in test/setup: {other}"),
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
fn context_capacity_error_reports_the_actual_injected_budget() {
    let mut messages = vec![msg("system", &"rules".repeat(4_000))];
    let tools = vec![serde_json::json!({
        "type": "function",
        "function": {
            "name": "enabled_tool",
            "description": "d".repeat(8_000),
            "parameters": {"type": "object", "properties": {}}
        }
    })];

    let error = prepare_for_request(&mut messages, 4_000, &tools, "ollama")
        .expect_err("mandatory context must exceed the configured window");
    let details = super::super::context_capacity_error::decode(&error)
        .expect("structured context capacity details");

    assert_eq!(details.context_window, 4_000);
    assert_eq!(details.max_input_tokens, 2_000);
    assert!(details.system_tokens > 0);
    assert_eq!(details.required_report_tokens, 0);
    assert!(details.tool_tokens > 0);
    assert_eq!(
        details.required_tokens,
        details
            .system_tokens
            .saturating_add(details.required_report_tokens)
            .saturating_add(details.tool_tokens)
    );
}

#[test]
fn context_capacity_error_counts_only_the_tools_given_to_the_request() {
    let mut messages = vec![msg("system", &"rules".repeat(4_000))];
    let enabled_tools = vec![serde_json::json!({
        "type": "function",
        "function": {
            "name": "only_enabled_tool",
            "description": "small",
            "parameters": {"type": "object", "properties": {}}
        }
    })];

    let error = prepare_for_request(&mut messages, 4_000, &enabled_tools, "ollama")
        .expect_err("system prompt alone must exceed the configured window");
    let details = super::super::context_capacity_error::decode(&error)
        .expect("structured context capacity details");

    assert_eq!(
        details.tool_tokens,
        token_estimate::estimate_tool_tokens(&enabled_tools) as u64
    );
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
fn payload_reduction_capacity_error_reports_the_real_context_window() {
    let mut messages = vec![msg("system", &"rules".repeat(20_000))];

    let error = reduce_after_payload_too_large(&mut messages, 128_000, &[], "ollama")
        .expect_err("mandatory context must exceed the reduced retry target");
    let details = super::super::context_capacity_error::decode(&error)
        .expect("structured context capacity details");

    assert_eq!(details.context_window, 128_000);
    assert!(details.max_input_tokens < details.context_window);
}

#[test]
fn payload_reduction_reports_known_counts_when_context_window_is_unknown() {
    let mut messages = vec![msg("system", &"rules".repeat(20_000))];

    let error = reduce_after_payload_too_large(&mut messages, 0, &[], "ollama")
        .expect_err("mandatory context must exceed the reduced retry target");
    let details = super::super::context_capacity_error::decode(&error)
        .expect("known counters remain structured");

    assert_eq!(details.context_window, 0);
    assert!(details.system_tokens > details.max_input_tokens);
    assert_eq!(details.required_tokens, details.system_tokens);
}

#[test]
fn codex_does_not_prune_reasoning_that_is_not_sent() {
    let mut messages = vec![msg("system", "rules"), msg("user", &"a".repeat(280_000))];
    messages.push(ChatMessage::assistant("recent answer".into(), Some("r".repeat(80_000)), None));
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

#[test]
fn invalid_tool_chain_salvages_assistant_text_and_reports_the_repair() {
    let mut messages = vec![
        assistant_with_calls(&["call-1"]),
        tool_message("wrong-call", "grep", "orphan"),
    ];
    messages[0].content = "useful assistant text".into();

    let report = prepare_for_request(&mut messages, 0, &[], "openai").unwrap();

    assert_eq!(report.repaired_tool_chains, 1);
    assert_eq!(report.dropped_tool_results, 1);
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].content, "useful assistant text");
    assert!(messages[0].tool_calls.is_none());
}

#[test]
fn pruning_keeps_a_contiguous_recent_suffix() {
    let mut messages = vec![
        msg("system", "rules"),
        msg("user", "old marker"),
        msg("assistant", &"middle".repeat(10_000)),
        msg("user", "recent marker"),
    ];

    prepare_for_request(&mut messages, 8_000, &[], "openai").unwrap();

    assert!(messages.iter().any(|message| message.content == "recent marker"));
    assert!(messages.iter().all(|message| message.content != "old marker"));
}

fn assistant_with_calls(ids: &[&str]) -> ChatMessage {
    ChatMessage::assistant(String::new(), None, Some(ids.iter().enumerate().map(|(index, id)| {
            super::super::types_ollama::ToolCallOllama {
                id: Some((*id).into()),
                extra_content: None,
                function: super::super::types_ollama::ToolCallFunction {
                    name: if index == 0 { "grep" } else { "glob" }.into(),
                    arguments: serde_json::json!({}),
                },
            }
        }).collect()))
}

fn tool_message(id: &str, name: &str, content: &str) -> ChatMessage {
    ChatMessage::tool(content.into(), Some(id.into()), Some(name.into()))
}
