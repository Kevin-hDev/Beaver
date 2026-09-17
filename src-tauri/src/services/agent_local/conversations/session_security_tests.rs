use super::*;
use serde_json::json;

#[test]
fn sanitizes_index_values_without_changing_shape() {
    let credential = ["xai", "-", &"A".repeat(24)].concat();
    let mut value = json!([{
        "id": "session-1",
        "name": credential,
        "message_count": 2
    }]);

    sanitize_index_value(&mut value);

    assert_eq!(value[0]["id"], "session-1");
    assert_eq!(value[0]["name"], "[REDACTED]");
    assert_eq!(value[0]["message_count"], 2);
}

#[test]
fn bounds_serialized_context_snapshots() {
    let mut value = json!({ "context_tokens": u32::MAX });

    bound_context_snapshot(&mut value);

    assert_eq!(value["context_tokens"], MAX_CONTEXT_SNAPSHOT_TOKENS);
}
