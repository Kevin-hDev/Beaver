use super::*;
use crate::services::agent_local::types_ollama::{ToolCallFunction, ToolCallOllama};

#[test]
fn token_estimate_includes_intermediate_answers_and_tool_calls() {
    let messages = vec![
        ChatMessage::assistant(
            "a".repeat(400),
            None,
            None,
            None,
            Some(vec![ToolCallOllama {
                id: Some("call-1".into()),
                extra_content: None,
                function: ToolCallFunction {
                    name: "list_dir".into(),
                    arguments: serde_json::json!({"path": "."}),
                },
            }]),
        ),
        ChatMessage::tool("README.md".into(), None, Some("list_dir".into())),
        ChatMessage::assistant("Terminé.".into(), None, None, None, None),
    ];

    assert!(generated_output_tokens(&messages) >= 100);
}
