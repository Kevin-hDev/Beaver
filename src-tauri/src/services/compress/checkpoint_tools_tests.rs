use super::checkpoint_messages::SelectedCheckpointMessage;
use super::checkpoint_selection::select;
use crate::services::agent_local::types_message::{ToolCallRequest, ToolCallRequestFunction};

use super::checkpoint_messages_tests::{limits, message};

fn tool_chain(
    count: usize,
    result: impl Fn(usize) -> String,
) -> Vec<crate::services::agent_local::types_session::AgentMessage> {
    let mut assistant = message("tools", "assistant", "");
    assistant.tool_calls = Some(
        (0..count)
            .map(|index| ToolCallRequest {
                id: format!("call-{index}"),
                extra_content: None,
                function: ToolCallRequestFunction {
                    name: "read_file".into(),
                    arguments: serde_json::json!({"path": format!("{index}.rs")}),
                },
            })
            .collect(),
    );
    let mut messages = vec![message("tools", "user", "inspect"), assistant];
    messages.extend((0..count).map(|index| {
        let mut tool = message("tools", "tool", result(index));
        tool.tool_call_id = Some(format!("call-{index}"));
        tool.tool_name = Some("read_file".into());
        tool
    }));
    messages.push(message("tools", "assistant", "done"));
    messages
}

#[test]
fn keeps_twenty_five_small_results_with_their_calls() {
    let source = tool_chain(25, |index| format!("result-{index}"));
    let selected = select(&source, limits(5_000, 5_000)).unwrap();
    assert_eq!(
        selected
            .messages
            .iter()
            .filter(|item| item.message().role == "tool")
            .count(),
        25
    );
    assert_eq!(
        selected
            .messages
            .iter()
            .filter(|item| item.message().tool_calls.is_some())
            .count(),
        1
    );
}

#[test]
fn bounds_a_large_result_and_keeps_existing_storage_reference() {
    let source = tool_chain(1, |_| {
        format!(
            "{}\n[Résultat complet disponible : tool-results/session/full.txt]",
            "x".repeat(40_000)
        )
    });
    let mut configured = limits(5_000, 5_000);
    configured.tool_tokens_per_result = 2_000;
    let original = source[2].content.clone();
    let selected = select(&source, configured).unwrap();
    let result = selected
        .messages
        .iter()
        .find(|item| item.message().role == "tool")
        .unwrap();
    assert!(matches!(
        result,
        SelectedCheckpointMessage::ToolResultExcerpt { .. }
    ));
    assert!(result.message().content.contains("[tool result excerpt]"));
    assert!(result
        .message()
        .content
        .contains("tool-results/session/full.txt"));
    assert_eq!(source[2].content, original);
}

#[test]
fn rejects_orphan_tool_results_and_caps_events_at_one_hundred() {
    let orphan = vec![
        message("orphan", "user", "q"),
        message("orphan", "tool", "result"),
    ];
    assert_eq!(
        select(&orphan, limits(5_000, 5_000)).unwrap_err(),
        "compression_checkpoint_invalid_tool_chain"
    );

    let source = tool_chain(101, |_| "ok".into());
    let selected = select(&source, limits(5_000, 5_000)).unwrap();
    assert_eq!(
        selected
            .messages
            .iter()
            .filter(|item| item.message().role == "tool")
            .count(),
        0
    );
}
