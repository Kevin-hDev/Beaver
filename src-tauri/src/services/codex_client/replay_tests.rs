use super::*;
use crate::services::agent_local::types_ollama::{ToolCallFunction, ToolCallOllama};

fn result_with_tool() -> StreamResult {
    StreamResult {
        tool_calls: vec![("read_file".to_string(), serde_json::json!({"path": "a"}))],
        tool_call_ids: vec!["call_1".to_string()],
        ..Default::default()
    }
}

#[test]
fn collector_attaches_bounded_codex_state_to_first_tool() {
    let reasoning = serde_json::json!({
        "type": "reasoning",
        "id": "rs_1",
        "encrypted_content": "opaque"
    });
    let function = serde_json::json!({
        "type": "function_call",
        "call_id": "call_1",
        "name": "read_file",
        "arguments": "{\"path\":\"a\"}"
    });
    let mut collector = ReplayCollector::default();
    collector.capture(&reasoning).unwrap();
    collector.capture(&function).unwrap();
    let mut result = result_with_tool();

    collector.attach(&mut result);

    let items = result.tool_call_extra_content[0]
        .as_ref()
        .unwrap()
        .pointer(CODEX_ITEMS_PATH)
        .unwrap();
    assert_eq!(items, &serde_json::json!([reasoning, function]));
}

#[test]
fn replay_rejects_state_for_another_tool_call() {
    let message = ChatMessage {
        role: "assistant".to_string(),
        content: String::new(),
        tool_calls: Some(vec![ToolCallOllama {
            id: Some("call_1".to_string()),
            extra_content: Some(serde_json::json!({
                "codex": {
                    "output_items": [{
                        "type": "function_call",
                        "call_id": "call_other",
                        "name": "read_file",
                        "arguments": "{}"
                    }]
                }
            })),
            function: ToolCallFunction {
                name: "read_file".to_string(),
                arguments: serde_json::json!({}),
            },
        }]),
        ..Default::default()
    };

    assert!(items_from_message(&message).is_none());
}
