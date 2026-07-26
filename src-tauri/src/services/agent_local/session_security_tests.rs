use super::*;
use crate::services::agent_local::types_ollama::{ToolCallFunction, ToolCallOllama};
use serde_json::json;

#[test]
fn sanitizes_model_content_and_tool_payloads() {
    let mut messages = vec![
        ChatMessage {
            role: "user".into(),
            content: "use gsk_1234567890abcdefghijkl".into(),
            reasoning_content: Some("xai-1234567890abcdefghijkl".into()),
            tool_calls: Some(vec![ToolCallOllama {
                id: Some("call-1".into()),
                extra_content: Some(json!({"access_token": "opaque-secret"})),
                function: ToolCallFunction {
                    name: "bash".into(),
                    arguments: json!({"command": "API_KEY=provider-secret"}),
                },
            }]),
            ..Default::default()
        },
        ChatMessage {
            role: "tool".into(),
            content: "MISTRAL_API_KEY=opaque-tool-secret".into(),
            ..Default::default()
        },
    ];

    sanitize_chat_messages(&mut messages);

    let serialized = serde_json::to_string(&messages).unwrap();
    for (index, secret) in [
        "gsk_1234567890abcdefghijkl",
        "xai-1234567890abcdefghijkl",
        "opaque-secret",
        "provider-secret",
        "opaque-tool-secret",
    ]
    .into_iter()
    .enumerate()
    {
        assert!(!serialized.contains(secret), "secret fixture {index}");
    }
    assert!(serialized.contains("[REDACTED]"));
}

#[test]
fn keeps_regular_source_code_in_user_messages() {
    let source = "let password = env::var(\"APP_PASSWORD\")?;";
    let mut messages = vec![ChatMessage {
        role: "user".into(),
        content: source.into(),
        ..Default::default()
    }];

    sanitize_chat_messages(&mut messages);

    assert_eq!(messages[0].content, source);
}

#[test]
fn sanitizes_serialized_sessions_without_dropping_fields() {
    let mut value = json!({
        "messages": [
            {"role": "user", "content": "let password = config.value;", "tokens": 4},
            {"role": "tool", "content": "token=old-secret", "tokens": 2}
        ],
        "provider": "ollama",
        "custom": [1, 2, 3]
    });
    sanitize_session_value(&mut value);
    assert_eq!(value["messages"].as_array().unwrap().len(), 2);
    assert_eq!(value["messages"][0]["tokens"], 4);
    assert_eq!(value["provider"], "ollama");
    assert_eq!(value["custom"], json!([1, 2, 3]));
    assert_eq!(
        value["messages"][0]["content"],
        "let password = config.value;"
    );
    assert_eq!(value["messages"][1]["content"], "token=[REDACTED]");
}
