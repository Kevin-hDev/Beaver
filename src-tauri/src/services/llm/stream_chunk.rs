use crate::services::provider_usage::RequestUsage;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum ParsedChunk {
    Thinking(String),
    Content(String),
    ToolCalls(Vec<Value>),
    Usage(RequestUsage),
}

pub fn parse(data: &str) -> Vec<ParsedChunk> {
    let chunk: Value = match serde_json::from_str(data) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    let mut out = Vec::new();
    if let Some(choice) = chunk["choices"].as_array().and_then(|a| a.first()) {
        parse_delta(&choice["delta"], &mut out);
    }
    let usage_value = chunk.get("usage").or_else(|| chunk.get("usageMetadata"));
    if let Some(usage) = usage_value.and_then(RequestUsage::from_json) {
        out.push(ParsedChunk::Usage(usage));
    }
    out
}

fn parse_delta(delta: &Value, out: &mut Vec<ParsedChunk>) {
    push_string(out, ParsedChunk::Thinking, &delta["reasoning_content"]);
    push_string(out, ParsedChunk::Thinking, &delta["reasoning"]);
    push_string(out, ParsedChunk::Thinking, &delta["thought"]);
    push_string(out, ParsedChunk::Thinking, &delta["thought_summary"]);
    parse_reasoning_details(&delta["reasoning_details"], out);
    parse_google_extra_content(&delta["extra_content"], out);
    parse_content(&delta["content"], out);
    if let Some(tcs) = delta["tool_calls"].as_array() {
        out.push(ParsedChunk::ToolCalls(tcs.clone()));
    }
}

fn parse_reasoning_details(value: &Value, out: &mut Vec<ParsedChunk>) {
    let Some(items) = value.as_array() else {
        return;
    };
    for item in items {
        for key in ["text", "summary"] {
            push_string(out, ParsedChunk::Thinking, &item[key]);
        }
    }
}

fn parse_google_extra_content(value: &Value, out: &mut Vec<ParsedChunk>) {
    let google = &value["google"];
    for key in ["thought", "thought_summary", "thinking"] {
        push_string(out, ParsedChunk::Thinking, &google[key]);
    }
    for key in ["thoughts", "thought_summaries"] {
        if let Some(items) = google[key].as_array() {
            for item in items {
                push_string(out, ParsedChunk::Thinking, item);
                push_string(out, ParsedChunk::Thinking, &item["text"]);
            }
        }
    }
}

fn parse_content(value: &Value, out: &mut Vec<ParsedChunk>) {
    if let Some(text) = value.as_str() {
        push_non_empty(out, ParsedChunk::Content, text);
        return;
    }
    let Some(items) = value.as_array() else {
        return;
    };
    for item in items {
        if item["thought"].as_bool() == Some(true) {
            parse_thinking_chunk(item, out);
            continue;
        }
        match item["type"].as_str().unwrap_or_default() {
            "thinking" | "thought" | "thought_summary" => parse_thinking_chunk(item, out),
            "text" => push_string(out, ParsedChunk::Content, &item["text"]),
            _ => {}
        }
    }
}

fn parse_thinking_chunk(item: &Value, out: &mut Vec<ParsedChunk>) {
    push_string(out, ParsedChunk::Thinking, &item["text"]);
    push_string(out, ParsedChunk::Thinking, &item["content"]);
    push_string(out, ParsedChunk::Thinking, &item["thinking"]);
    if let Some(inner) = item["thinking"].as_array() {
        for chunk in inner {
            push_string(out, ParsedChunk::Thinking, &chunk["text"]);
        }
    }
}

fn push_string<F>(out: &mut Vec<ParsedChunk>, build: F, value: &Value)
where
    F: Fn(String) -> ParsedChunk,
{
    if let Some(text) = value.as_str() {
        push_non_empty(out, build, text);
    }
}

fn push_non_empty<F>(out: &mut Vec<ParsedChunk>, build: F, text: &str)
where
    F: Fn(String) -> ParsedChunk,
{
    if !text.is_empty() {
        out.push(build(text.to_string()));
    }
}
