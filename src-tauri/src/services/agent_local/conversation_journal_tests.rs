use super::conversation_journal::validate_tool_results;
use super::types_ollama::ChatMessage;

#[test]
fn journal_rejects_missing_duplicate_and_reordered_tool_results() {
    let expected = vec!["call-a".to_string(), "call-b".to_string()];
    assert!(validate_tool_results(&[tool("call-a"), tool("call-b")], &expected).is_ok());
    assert!(validate_tool_results(&[tool("call-a")], &expected).is_err());
    assert!(validate_tool_results(&[tool("call-a"), tool("call-a")], &expected).is_err());
    assert!(validate_tool_results(&[tool("call-b"), tool("call-a")], &expected).is_err());
}

fn tool(id: &str) -> ChatMessage { ChatMessage::tool("result".into(), Some(id.into()), Some("bash".into())) }
