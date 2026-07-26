use super::transport::extract_tool_result;
use serde_json::json;

#[test]
fn connector_errors_are_generic() {
    for response in [
        json!({"error": {"message": "secret"}}),
        json!({"error": {}}),
    ] {
        assert_eq!(
            extract_tool_result(&response).unwrap_err(),
            "erreur MCP retournée par le connecteur"
        );
    }
}

#[test]
fn text_results_are_extracted() {
    let one = json!({"result": {"content": [{"text": "hello"}]}});
    assert_eq!(extract_tool_result(&one).unwrap(), "hello");

    let many = json!({"result": {"content": [{"text": "a"}, {"text": "b"}]}});
    assert_eq!(extract_tool_result(&many).unwrap(), "a\nb");
}

#[test]
fn structured_results_are_serialized() {
    let response = json!({"result": {"data": 42}});
    assert!(extract_tool_result(&response).unwrap().contains("42"));
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
