use serde_json::Value;

pub(super) fn chunks(event: &Value) -> Vec<Value> {
    event
        .pointer("/choices/0/delta/content")
        .or_else(|| event.pointer("/choices/0/message/content"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}
