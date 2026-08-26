use serde_json::Value;

pub(super) fn parts(event: &Value) -> Vec<Value> {
    event
        .pointer("/candidates/0/content/parts")
        .or_else(|| event.pointer("/candidate/content/parts"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}
