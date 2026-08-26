use serde_json::Value;

pub(super) fn fragments(event: &Value) -> Vec<&str> {
    let Some(delta) = event.pointer("/choices/0/delta") else {
        return event
            .pointer("/choices/0/message")
            .map_or_else(Vec::new, from_message);
    };
    from_message(delta)
}

fn from_message(message: &Value) -> Vec<&str> {
    [
        "reasoning_content",
        "reasoning",
        "thought",
        "thought_summary",
    ]
    .into_iter()
    .filter_map(|key| message.get(key).and_then(Value::as_str))
    .collect()
}
