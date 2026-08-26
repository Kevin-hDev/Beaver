use super::{completed_item, final_items, tool_link, ResponseItemError};
use serde_json::json;

#[test]
fn completed_item_preserves_reasoning_bytes_and_function_link() {
    let item = json!({
        "type": "reasoning",
        "id": "rs_1",
        "encrypted_content": "opaque-provider-value"
    });
    let event = json!({"type": "response.output_item.done", "item": item});

    let captured = completed_item(&event).unwrap().unwrap();

    assert_eq!(
        serde_json::to_vec(&captured).unwrap(),
        serde_json::to_vec(&event["item"]).unwrap()
    );
    let function = json!({"type":"function_call","call_id":"call_1","name":"lookup"});
    assert_eq!(
        tool_link(&function).unwrap().unwrap().provider_call_id,
        "call_1"
    );
}

#[test]
fn completed_item_rejects_unknown_output_instead_of_dropping_it() {
    let event = json!({
        "type": "response.output_item.done",
        "item": {"type": "web_search_call", "id": "ws_1"}
    });

    assert_eq!(
        completed_item(&event),
        Err(ResponseItemError::UnsupportedItem)
    );
}

#[test]
fn completed_response_uses_final_output_only_when_needed() {
    let completed = json!({
        "type": "response.completed",
        "response": {"output": [
            {"type":"reasoning","encrypted_content":"opaque"},
            {"type":"message","content":[]}
        ]}
    });

    assert_eq!(final_items(&completed).unwrap().len(), 2);
    assert!(final_items(&json!({"type":"response.created"}))
        .unwrap()
        .is_empty());
}
