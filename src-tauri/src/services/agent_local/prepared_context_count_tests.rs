use serde_json::json;

use super::context_usage_record::{ContextCountCoverage, ContextCountSource};
use super::prepared_context_count::{anthropic, chat_completions, ollama, responses};

#[test]
fn counts_only_semantic_fields_for_each_http_family() {
    let chat = json!({
        "model": "ignored", "stream": true,
        "messages": [{"role": "user", "content": "hello"}],
        "tools": [{"type": "function", "function": {"name": "lookup"}}]
    });
    let responses_body = json!({
        "instructions": "be concise", "input": [{"role": "user", "content": "hello"}],
        "tools": [{"type": "function", "name": "lookup"}], "stream": true
    });
    let anthropic_body = json!({
        "system": [{"type": "text", "text": "be concise"}],
        "messages": [{"role": "user", "content": "hello"}],
        "tools": [{"name": "lookup"}], "max_tokens": 1024
    });

    for count in [
        chat_completions(&chat),
        responses(&responses_body),
        anthropic(&anthropic_body),
    ] {
        assert!(count.tokens.unwrap() > 0);
        assert_eq!(count.tokens, count.capacity_tokens);
        assert_eq!(count.source, Some(ContextCountSource::Heuristic));
        assert_eq!(count.coverage, ContextCountCoverage::Complete);
    }

    let mut technical_change = chat.clone();
    technical_change["stream"] = false.into();
    technical_change["metadata"] = json!({"route": "elsewhere"});
    assert_eq!(chat_completions(&chat), chat_completions(&technical_change));
}

#[test]
fn final_ollama_wire_includes_messages_tools_images_and_replay() {
    let without_replay = json!({
        "model": "deepseek-r1:latest", "stream": true,
        "messages": [{"role": "user", "content": "hello", "images": ["aGVsbG8="]}],
        "tools": [{"type": "function", "function": {"name": "lookup"}}]
    });
    let with_replay = json!({
        "model": "deepseek-r1:latest", "stream": true,
        "messages": [
            {"role": "assistant", "content": "", "thinking": "remember this"},
            {"role": "user", "content": "hello", "images": ["aGVsbG8="]}
        ],
        "tools": [{"type": "function", "function": {"name": "lookup"}}]
    });
    let without_replay = ollama(&without_replay);
    let with_replay = ollama(&with_replay);

    assert!(with_replay.tokens.unwrap() > without_replay.tokens.unwrap());
    assert_eq!(with_replay.coverage, ContextCountCoverage::Partial);
    assert!(with_replay.capacity_tokens.is_some());
}

#[test]
fn bounded_media_has_capacity_but_remote_and_opaque_blocks_do_not() {
    let inline = json!({"messages": [{"role": "user", "content": [
        {"type": "text", "text": "inspect"},
        {"type": "image_url", "image_url": {"url": "data:image/png;base64,aGVsbG8="}}
    ]}]});
    let remote = json!({"messages": [{"role": "user", "content": [
        {"type": "image_url", "image_url": {"url": "https://example.test/image.png"}}
    ]}]});
    let opaque = json!({
        "instructions": "continue",
        "input": [{"type": "reasoning", "encrypted_content": "opaque"}]
    });
    let signed_tool_call = json!({"messages": [{
        "role": "assistant",
        "tool_calls": [{
            "extra_content": {"google": {"thought_signature": "opaque"}},
            "function": {"name": "lookup", "arguments": "{}"}
        }]
    }]});
    let anthropic_image = json!({"messages": [{"role": "user", "content": [{
        "type": "image", "source": {"type": "base64", "media_type": "image/png", "data": "aGVsbG8="}
    }]}]});

    let inline_count = chat_completions(&inline);
    assert_eq!(inline_count.coverage, ContextCountCoverage::Partial);
    assert!(inline_count.capacity_tokens.unwrap() >= inline_count.tokens.unwrap());
    assert_eq!(anthropic(&anthropic_image).coverage, ContextCountCoverage::Partial);

    for count in [
        chat_completions(&remote),
        responses(&opaque),
        chat_completions(&signed_tool_call),
    ] {
        assert_eq!(count.coverage, ContextCountCoverage::Unknown);
        assert_eq!(count.tokens, None);
        assert_eq!(count.capacity_tokens, None);
        assert_eq!(count.source, None);
    }
}

#[test]
fn tool_schema_and_arguments_may_use_opaque_field_names_as_plain_data() {
    let chat = json!({
        "messages": [{"role": "user", "content": "sign it"}],
        "tools": [{
            "type": "function",
            "function": {
                "name": "sign_document",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "signature": {"type": "string"},
                        "images": {"type": "array"}
                    }
                }
            }
        }]
    });
    let ollama_payload = json!({
        "messages": [{
            "role": "assistant",
            "content": "",
            "tool_calls": [{
                "function": {
                    "name": "sign_document",
                    "arguments": {"signature": "visible", "images": ["one"]}
                }
            }]
        }],
        "tools": chat["tools"].clone()
    });

    for count in [chat_completions(&chat), ollama(&ollama_payload)] {
        assert_eq!(count.coverage, ContextCountCoverage::Complete);
        assert!(count.tokens.is_some());
        assert_eq!(count.tokens, count.capacity_tokens);
    }
}
