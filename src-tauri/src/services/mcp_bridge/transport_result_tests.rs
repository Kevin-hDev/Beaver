use super::transport::{extract_tool_result, McpCallError};
use serde_json::json;

#[test]
fn connector_errors_are_generic() {
    for response in [
        json!({"error": {"message": "secret"}}),
        json!({"error": {}}),
    ] {
        assert_eq!(
            extract_tool_result(&response).unwrap_err(),
            McpCallError::Server
        );
    }
}

#[test]
fn text_results_are_extracted() {
    let one = json!({"result": {"content": [{"text": "hello"}]}});
    assert_eq!(extract_tool_result(&one).unwrap().content, "hello");

    let many = json!({"result": {"content": [{"text": "a"}, {"text": "b"}]}});
    assert_eq!(extract_tool_result(&many).unwrap().content, "a\nb");
}

#[test]
fn structured_results_are_serialized() {
    let response = json!({"result": {"data": 42}});
    assert!(extract_tool_result(&response)
        .unwrap()
        .content
        .contains("42"));
}

#[test]
fn tool_level_errors_are_preserved() {
    let response = json!({
        "result": {"content": [{"text": "invalid query"}], "isError": true}
    });
    let result = extract_tool_result(&response).expect("valid MCP tool error");

    assert!(result.is_error);
    assert_eq!(result.content, "invalid query");
}

#[test]
fn malformed_error_flag_is_rejected() {
    let response = json!({"result": {"content": [], "isError": "yes"}});
    assert!(extract_tool_result(&response).is_err());
}

#[test]
fn empty_results_are_rejected() {
    assert!(extract_tool_result(&json!({})).is_err());
}

#[test]
fn excessive_content_collection_is_rejected() {
    let content = vec![json!({"text": "x"}); 257];
    assert!(extract_tool_result(&json!({"result": {"content": content}})).is_err());
}
