use super::*;
use serde_json::json;

#[test]
fn chat_ignores_explicit_agentic_tools() {
    let definitions = definitions_for_mode(
        true,
        &[json!({"type": "function", "function": {"name": "bash"}})],
    );
    let names = tool_catalog::tool_names(&definitions);

    assert_eq!(names, vec!["web_search", "web_fetch"]);
}
