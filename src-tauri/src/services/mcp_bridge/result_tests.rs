use super::result::complete;
use super::transport::McpCallError;
use serde_json::json;

#[test]
fn structured_scalar_and_array_remain_visible() {
    for structured in [json!(42), json!(["one", "two"])] {
        let reply = json!({"resultType": "complete", "structuredContent": structured});
        let result = complete(&reply).expect("valid final result");
        let rendered: serde_json::Value =
            serde_json::from_str(&result.content).expect("structured JSON preserved");
        assert_eq!(rendered["structuredContent"], structured);
    }
}

#[test]
fn non_text_blocks_do_not_disappear_beside_text() {
    let reply = json!({"content": [
        {"type": "text", "text": "summary"},
        {"type": "image", "text": "caption", "data": "bounded-data"}
    ]});
    let result = complete(&reply).expect("valid final result");
    assert!(result.content.contains("summary"));
    assert!(result.content.contains("bounded-data"));
}

#[test]
fn empty_content_is_a_valid_final_result() {
    let result =
        complete(&json!({"resultType": "complete", "content": []})).expect("empty final result");
    assert!(!result.is_error);
    assert!(result.content.contains("\"content\": []"));
}

#[test]
fn non_object_or_unknown_result_type_is_rejected() {
    for reply in [json!(null), json!([]), json!({"resultType": "task"})] {
        assert_eq!(complete(&reply), Err(McpCallError::InvalidResponse));
    }
}
