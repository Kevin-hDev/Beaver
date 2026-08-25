use super::*;
use crate::services::agent_local::types_ollama::{ToolCallFunction, ToolCallOllama};

#[test]
fn groups_a_call_with_all_of_its_results_without_cloning_messages() {
    let assistant = assistant_call("call-1", "kept text");
    let result = tool_result("call-1");
    let units = atomic_units(vec![&assistant, &result]);

    assert_eq!(units.len(), 1);
    assert!(units[0].valid);
    assert!(units[0].is_tool_chain);
    assert_eq!(units[0].messages.len(), 2);
}

#[test]
fn repair_salvages_assistant_text_and_drops_mismatched_result() {
    let mut messages = vec![assistant_call("call-1", "kept text"), tool_result("call-2")];

    let report = repair_invalid_history(&mut messages);

    assert_eq!(report.repaired_tool_chains, 1);
    assert_eq!(report.dropped_tool_results, 1);
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].content, "kept text");
    assert!(messages[0].tool_calls.is_none());
}

#[test]
fn repair_drops_an_orphan_tool_result() {
    let mut messages = vec![tool_result("call-1")];

    let report = repair_invalid_history(&mut messages);

    assert_eq!(report.dropped_tool_results, 1);
    assert!(messages.is_empty());
}

#[test]
fn repair_rejects_a_matching_id_on_a_non_tool_message() {
    let mut impostor = tool_result("call-1");
    impostor.role = "user".into();
    let mut messages = vec![assistant_call("call-1", "kept text"), impostor];

    let report = repair_invalid_history(&mut messages);

    assert_eq!(report.repaired_tool_chains, 1);
    assert_eq!(messages.len(), 2);
    assert!(messages[0].tool_calls.is_none());
    assert_eq!(messages[1].role, "user");
}

#[test]
fn repair_rejects_duplicate_call_ids() {
    let mut assistant = assistant_call("call-1", "kept text");
    let duplicate = assistant.tool_calls.as_ref().unwrap()[0].clone();
    assistant.tool_calls.as_mut().unwrap().push(duplicate);
    let mut messages = vec![assistant, tool_result("call-1"), tool_result("call-1")];

    let report = repair_invalid_history(&mut messages);

    assert_eq!(report.repaired_tool_chains, 1);
    assert_eq!(report.dropped_tool_results, 2);
    assert_eq!(messages.len(), 1);
    assert!(messages[0].tool_calls.is_none());
}

#[test]
fn empty_ids_are_normalized_but_empty_tool_names_are_rejected() {
    let mut assistant = assistant_call("call-1", "kept text");
    assistant.tool_calls.as_mut().unwrap()[0].id = Some(String::new());
    let mut result = tool_result("call-1");
    result.tool_call_id = Some(String::new());
    let mut valid_messages = vec![assistant, result];
    assert_eq!(repair_invalid_history(&mut valid_messages), HistoryRepairReport::default());

    let mut assistant = assistant_call("call-1", "kept text");
    assistant.tool_calls.as_mut().unwrap()[0].function.name.clear();
    let mut invalid_messages = vec![assistant, tool_result("call-1")];
    let report = repair_invalid_history(&mut invalid_messages);

    assert_eq!(report.repaired_tool_chains, 1);
    assert_eq!(invalid_messages.len(), 1);
    assert!(invalid_messages[0].tool_calls.is_none());
}

fn assistant_call(id: &str, content: &str) -> ChatMessage {
    ChatMessage::assistant(content.into(), None, Some(vec![ToolCallOllama {
            id: Some(id.into()),
            extra_content: None,
            function: ToolCallFunction {
                name: "grep".into(),
                arguments: serde_json::json!({}),
            },
        }]))
}

fn tool_result(id: &str) -> ChatMessage {
    ChatMessage::tool("ok".into(), Some(id.into()), Some("grep".into()))
}
