use super::stream_chunk::{self, ParsedChunk};
use serde_json::json;

fn parse(value: serde_json::Value) -> Vec<ParsedChunk> {
    stream_chunk::parse(&value.to_string())
}

#[test]
fn parses_openai_style_reasoning_fields() {
    let chunks = parse(json!({
        "choices": [{ "delta": { "reasoning_content": "think ", "content": "answer" } }]
    }));
    assert_eq!(
        chunks,
        vec![
            ParsedChunk::Thinking("think ".into()),
            ParsedChunk::Content("answer".into())
        ]
    );
}

#[test]
fn parses_openrouter_reasoning_details() {
    let chunks = parse(json!({
        "choices": [{
            "delta": {
                "reasoning_details": [{
                    "type": "reasoning.text",
                    "text": "step",
                    "signature": null
                }]
            }
        }]
    }));
    assert_eq!(chunks, vec![ParsedChunk::Thinking("step".into())]);
}

#[test]
fn parses_mistral_content_chunks() {
    let chunks = parse(json!({
        "choices": [{
            "delta": {
                "content": [
                    { "type": "thinking", "thinking": [{ "type": "text", "text": "calc" }] },
                    { "type": "text", "text": "done" }
                ]
            }
        }]
    }));
    assert_eq!(
        chunks,
        vec![
            ParsedChunk::Thinking("calc".into()),
            ParsedChunk::Content("done".into())
        ]
    );
}

#[test]
fn parses_gemini_extra_content_thoughts_without_signature() {
    let chunks = parse(json!({
        "choices": [{
            "delta": {
                "extra_content": {
                    "google": {
                        "thought_summary": "summary",
                        "thought_signature": "secret-signature"
                    }
                }
            }
        }]
    }));
    assert_eq!(chunks, vec![ParsedChunk::Thinking("summary".into())]);
}

#[test]
fn parses_gemini_thought_summary_delta() {
    let chunks = parse(json!({
        "choices": [{ "delta": { "thought_summary": "checking", "content": "answer" } }]
    }));
    assert_eq!(
        chunks,
        vec![
            ParsedChunk::Thinking("checking".into()),
            ParsedChunk::Content("answer".into())
        ]
    );
}

#[test]
fn parses_thought_content_parts() {
    let chunks = parse(json!({
        "choices": [{
            "delta": {
                "content": [
                    { "type": "thought", "text": "hidden" },
                    { "type": "text", "text": "visible" }
                ]
            }
        }]
    }));
    assert_eq!(
        chunks,
        vec![
            ParsedChunk::Thinking("hidden".into()),
            ParsedChunk::Content("visible".into())
        ]
    );
}

#[test]
fn returns_tool_calls_and_usage() {
    let chunks = parse(json!({
        "choices": [{ "delta": { "tool_calls": [{ "id": "a" }] } }],
        "usage": { "completion_tokens": 3, "prompt_tokens": 2 }
    }));
    assert_eq!(
        chunks,
        vec![
            ParsedChunk::ToolCalls(vec![json!({ "id": "a" })]),
            ParsedChunk::Usage(crate::services::provider_usage::RequestUsage {
                input_tokens: Some(2),
                output_tokens: Some(3),
                total_tokens: Some(5),
                ..Default::default()
            })
        ]
    );
}

#[test]
fn usage_missing_fields_stays_unavailable() {
    let chunks = parse(json!({ "usage": { "prompt_tokens": 2 } }));
    assert_eq!(
        chunks,
        vec![ParsedChunk::Usage(
            crate::services::provider_usage::RequestUsage {
                input_tokens: Some(2),
                ..Default::default()
            }
        )]
    );
}

#[test]
fn missing_usage_emits_no_usage_chunk() {
    let chunks = parse(json!({ "choices": [{ "delta": { "content": "answer" } }] }));
    assert_eq!(chunks, vec![ParsedChunk::Content("answer".into())]);
}
