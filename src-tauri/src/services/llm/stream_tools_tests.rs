use super::*;
use serde_json::json;

#[test]
fn accumulates_fragmented_arguments_openai_style() {
    let mut acc = ToolCallAccumulator::new();
    acc.ingest(&[json!({
        "index": 0, "id": "call_1", "type": "function",
        "function": { "name": "web_search", "arguments": "" }
    })]);
    acc.ingest(&[json!({
        "index": 0, "function": { "arguments": "{\"query\":" }
    })]);
    acc.ingest(&[json!({
        "index": 0, "function": { "arguments": " \"rust tauri 2\"}" }
    })]);
    let (calls, ids, extra) = acc.finalize();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].0, "web_search");
    assert_eq!(calls[0].1["query"], "rust tauri 2");
    assert_eq!(ids[0], "call_1");
    assert_eq!(extra, vec![None]);
}

#[test]
fn reports_pending_tool_call_before_finalize() {
    let mut acc = ToolCallAccumulator::new();
    assert!(!acc.has_pending());
    acc.ingest(&[json!({
        "index": 0, "id": "call_1", "type": "function",
        "function": { "name": "web_search", "arguments": "{\"query\":" }
    })]);
    assert!(acc.has_pending());
}

#[test]
fn accumulates_complete_tool_call_groq_style() {
    let mut acc = ToolCallAccumulator::new();
    acc.ingest(&[json!({
        "index": 0, "id": "call_x", "type": "function",
        "function": {
            "name": "read_file",
            "arguments": "{\"path\": \"/tmp/x\"}"
        }
    })]);
    let (calls, _, _) = acc.finalize();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].1["path"], "/tmp/x");
}

#[test]
fn accumulates_parallel_tool_calls_by_index() {
    let mut acc = ToolCallAccumulator::new();
    acc.ingest(&[
        json!({
            "index": 0, "id": "a", "type": "function",
            "function": { "name": "f1", "arguments": "{\"x\":1}" }
        }),
        json!({
            "index": 1, "id": "b", "type": "function",
            "function": { "name": "f2", "arguments": "{\"y\":2}" }
        }),
    ]);
    let (calls, ids, _) = acc.finalize();
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0].0, "f1");
    assert_eq!(calls[1].0, "f2");
    assert_eq!(ids, vec!["a".to_string(), "b".to_string()]);
}

#[test]
fn tolerates_gemini_missing_index() {
    let mut acc = ToolCallAccumulator::new();
    acc.ingest(&[json!({
        "id": "0", "type": "function",
        "function": { "name": "g1", "arguments": "{\"q\":\"x\"}" }
    })]);
    let (calls, _, _) = acc.finalize();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].0, "g1");
}

#[test]
fn preserves_gemini_tool_call_extra_content() {
    let mut acc = ToolCallAccumulator::new();
    acc.ingest(&[json!({
        "index": 0,
        "id": "function-call-1",
        "type": "function",
        "extra_content": { "google": { "thought_signature": "sig-a" } },
        "function": { "name": "read_file", "arguments": "{\"path\":\"a\"}" }
    })]);
    let (_, _, extra) = acc.finalize();
    assert_eq!(
        extra[0],
        Some(json!({ "google": { "thought_signature": "sig-a" } }))
    );
}

#[test]
fn ignores_tool_calls_over_limit() {
    let mut acc = ToolCallAccumulator::new();
    for index in 0..(MAX_TOOL_CALLS + 4) {
        acc.ingest(&[json!({
            "index": index,
            "id": format!("call_{index}"),
            "type": "function",
            "function": { "name": "read_file", "arguments": "{\"path\":\"a\"}" }
        })]);
    }

    let (calls, ids, _) = acc.finalize();
    assert_eq!(calls.len(), MAX_TOOL_CALLS);
    assert_eq!(ids.len(), MAX_TOOL_CALLS);
}

#[test]
fn drops_tool_call_when_arguments_exceed_limit() {
    let mut acc = ToolCallAccumulator::new();
    acc.ingest(&[json!({
        "index": 0,
        "id": "call_large",
        "type": "function",
        "function": { "name": "read_file", "arguments": "{" }
    })]);
    acc.ingest(&[json!({
        "index": 0,
        "function": { "arguments": "x".repeat(MAX_TOOL_ARGUMENT_CHARS + 1) }
    })]);

    let (calls, ids, extra) = acc.finalize();
    assert!(calls.is_empty());
    assert!(ids.is_empty());
    assert!(extra.is_empty());
}

#[test]
fn drops_tool_call_when_extra_content_exceeds_limit() {
    let mut acc = ToolCallAccumulator::new();
    acc.ingest(&[json!({
        "index": 0,
        "id": "call_extra",
        "type": "function",
        "extra_content": { "google": { "thought_signature": "x".repeat(MAX_EXTRA_CONTENT_CHARS + 1) } },
        "function": { "name": "read_file", "arguments": "{\"path\":\"a\"}" }
    })]);

    let (calls, _, _) = acc.finalize();
    assert!(calls.is_empty());
}
