use super::*;
use crate::services::agent_local::types_ollama::{ToolCallFunction, ToolCallOllama};

fn user_with_png() -> ChatMessage {
    ChatMessage::user("Regarde".into()).with_images(vec!["iVBORw0KGgo=".into()])
}

fn policy(provider: &str, model: &str) -> super::super::route_profile::MessageWirePolicy {
    super::super::route_profile::payload_policy(provider, model)
        .unwrap()
        .message
}

#[test]
fn openai_style_image_uses_object_url() {
    let out = message_to_openai(&user_with_png(), policy("google", "gemini-3.5-flash"));
    assert_eq!(out["role"], "user");
    assert_eq!(out["content"][0]["type"], "text");
    assert_eq!(out["content"][1]["type"], "image_url");
    assert_eq!(
        out["content"][1]["image_url"]["url"],
        "data:image/png;base64,iVBORw0KGgo="
    );
}

#[test]
fn mistral_image_uses_string_url() {
    let out = message_to_openai(&user_with_png(), policy("mistral", "mistral-large"));
    assert_eq!(out["content"][1]["type"], "image_url");
    assert_eq!(
        out["content"][1]["image_url"],
        "data:image/png;base64,iVBORw0KGgo="
    );
}

#[test]
fn assistant_tool_call_preserves_extra_content() {
    let msg = ChatMessage::assistant(
        String::new(),
        None,
        None,
        None,
        Some(vec![ToolCallOllama {
            id: Some("function-call-1".into()),
            extra_content: Some(serde_json::json!({
                "google": { "thought_signature": "sig-a" }
            })),
            function: ToolCallFunction {
                name: "read_file".into(),
                arguments: serde_json::json!({ "path": "a" }),
            },
        }]),
    );

    let out = message_to_openai(&msg, policy("google", "gemini-3.5-flash"));
    assert_eq!(
        out["tool_calls"][0]["extra_content"],
        serde_json::json!({ "google": { "thought_signature": "sig-a" } })
    );
}

#[test]
fn deepseek_tool_call_keeps_non_null_assistant_content() {
    let msg = ChatMessage::assistant(
        String::new(),
        None,
        None,
        None,
        Some(vec![ToolCallOllama {
            id: Some("call-1".into()),
            extra_content: None,
            function: ToolCallFunction {
                name: "lookup".into(),
                arguments: serde_json::json!({}),
            },
        }]),
    );

    let out = message_to_openai(&msg, policy("deepseek", "deepseek-v4-flash"));

    assert_eq!(out["content"], "");
}
