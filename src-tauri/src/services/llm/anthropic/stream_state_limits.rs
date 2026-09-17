use serde_json::Value;

pub(super) fn accepts(current: usize, event: &Value, maximum: usize) -> bool {
    current.saturating_add(event_text_bytes(event)) <= maximum
}

fn event_text_bytes(event: &Value) -> usize {
    match event.get("type").and_then(Value::as_str) {
        Some("content_block_start") => event
            .pointer("/content_block/text")
            .or_else(|| event.pointer("/content_block/thinking"))
            .and_then(Value::as_str)
            .map_or(0, str::len),
        Some("content_block_delta") => event
            .pointer("/delta/text")
            .or_else(|| event.pointer("/delta/thinking"))
            .and_then(Value::as_str)
            .map_or(0, str::len),
        _ => 0,
    }
}
