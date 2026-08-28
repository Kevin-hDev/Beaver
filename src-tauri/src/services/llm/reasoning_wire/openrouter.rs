use serde_json::Value;

pub(super) fn details(event: &Value) -> Vec<Value> {
    event
        .pointer("/choices/0/delta/reasoning_details")
        .or_else(|| event.pointer("/choices/0/message/reasoning_details"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}
