use super::*;
use serde_json::json;

#[test]
fn sanitizes_model_content_and_tool_payloads() {
    let untrusted_user: ChatMessage = serde_json::from_value(json!({
        "role": "user",
        "content": "use gsk_1234567890abcdefghijkl",
        "reasoning_content": "xai-1234567890abcdefghijkl",
        "tool_calls": [{
            "id": "call-1",
            "extra_content": {"access_token": "opaque-secret"},
            "function": {
                "name": "bash",
                "arguments": {"command": "API_KEY=provider-secret"}
            }
        }]
    }))
    .unwrap();
    let mut messages = vec![
        untrusted_user,
        ChatMessage::tool("MISTRAL_API_KEY=opaque-tool-secret".into(), None, None),
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
    let mut messages = vec![ChatMessage::user(source.into())];

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

#[test]
fn bounds_serialized_context_snapshots() {
    let mut value = json!({ "context_tokens": u32::MAX });

    sanitize_session_value(&mut value);

    assert_eq!(value["context_tokens"], MAX_CONTEXT_SNAPSHOT_TOKENS);
}
