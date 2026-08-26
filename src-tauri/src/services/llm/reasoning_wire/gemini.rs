use serde_json::Value;

pub(super) fn parts(event: &Value) -> Vec<Value> {
    let Some(message) = event
        .pointer("/choices/0/delta")
        .or_else(|| event.pointer("/choices/0/message"))
    else {
        return Vec::new();
    };
    let mut parts = Vec::new();
    if !message["extra_content"].is_null() {
        parts.push(serde_json::json!({
            "extra_content": message["extra_content"].clone()
        }));
    }
    if let Some(tool_calls) = message["tool_calls"].as_array() {
        for (position, tool_call) in tool_calls.iter().enumerate() {
            if tool_call["extra_content"].is_null() {
                continue;
            }
            parts.push(serde_json::json!({
                "tool_call": {
                    "index": tool_call["index"].as_u64().unwrap_or(position as u64),
                    "extra_content": tool_call["extra_content"].clone()
                }
            }));
        }
    }
    parts
}
