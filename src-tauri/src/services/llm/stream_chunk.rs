use crate::services::provider_usage::{RequestUsage, UsageContext};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum ParsedChunk {
    Thinking(String),
    Content(String),
    ToolCalls(Vec<Value>),
    Usage(RequestUsage),
    GenerationDuration(u64),
    ProviderError(Option<u16>),
}

#[cfg(test)]
pub fn parse(data: &str) -> Vec<ParsedChunk> {
    parse_with_context(data, UsageContext::chat("unknown", "unknown"))
}

pub fn parse_with_context(data: &str, context: UsageContext<'_>) -> Vec<ParsedChunk> {
    let chunk: Value = match serde_json::from_str(data) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    parse_value_with_context(&chunk, context)
}

pub fn parse_value_with_context(chunk: &Value, context: UsageContext<'_>) -> Vec<ParsedChunk> {
    if chunk.get("error").is_some()
        || chunk
            .pointer("/choices/0/finish_reason")
            .and_then(Value::as_str)
            == Some("error")
    {
        return vec![ParsedChunk::ProviderError(
            chunk
                .pointer("/error/code")
                .and_then(Value::as_u64)
                .and_then(|code| code.try_into().ok()),
        )];
    }
    let mut out = Vec::new();
    if let Some(choice) = chunk["choices"].as_array().and_then(|a| a.first()) {
        parse_delta(&choice["delta"], &mut out);
    }
    if let Some(usage) = parse_usage(&chunk, context) {
        out.push(ParsedChunk::Usage(usage));
    }
    let completion_seconds = chunk
        .pointer("/usage/completion_time")
        .or_else(|| chunk.pointer("/x_groq/usage/completion_time"))
        .or_else(|| chunk.pointer("/time_info/completion_time"))
        .and_then(Value::as_f64);
    if let Some(duration_ns) = completion_seconds
        .and_then(crate::services::agent_local::generation_metrics::seconds_to_duration_ns)
    {
        out.push(ParsedChunk::GenerationDuration(duration_ns));
    }
    out
}

pub(super) fn provider_error_code(status: Option<u16>) -> &'static str {
    match status {
        Some(429) => "rate_limit",
        Some(408 | 500 | 502 | 503 | 504) => "provider_temporarily_unavailable",
        _ => "provider_request_rejected",
    }
}

fn parse_usage(chunk: &Value, context: UsageContext<'_>) -> Option<RequestUsage> {
    let choice_usage = (context.canonical_provider_id == "moonshot")
        .then(|| chunk.pointer("/choices/0/usage"))
        .flatten();
    let groq_usage = (context.canonical_provider_id == "groq")
        .then(|| chunk.pointer("/x_groq/usage"))
        .flatten();
    [
        chunk.get("usage"),
        chunk.get("usageMetadata"),
        choice_usage,
        groq_usage,
    ]
    .into_iter()
    .flatten()
    .filter(|value| !value.is_null())
    .find_map(|value| RequestUsage::from_json_with_context(value, context))
}

fn parse_delta(delta: &Value, out: &mut Vec<ParsedChunk>) {
    push_string(out, ParsedChunk::Thinking, &delta["reasoning_content"]);
    push_string(out, ParsedChunk::Thinking, &delta["reasoning"]);
    push_string(out, ParsedChunk::Thinking, &delta["thought"]);
    push_string(out, ParsedChunk::Thinking, &delta["thought_summary"]);
    append_openrouter_display(&delta["reasoning_details"], out);
    append_google_display(&delta["extra_content"], out);
    parse_content(&delta["content"], out);
    if let Some(tcs) = delta["tool_calls"].as_array() {
        out.push(ParsedChunk::ToolCalls(tcs.clone()));
    }
}

fn append_openrouter_display(value: &Value, out: &mut Vec<ParsedChunk>) {
    let Some(items) = value.as_array() else {
        return;
    };
    for item in items {
        for key in ["text", "summary"] {
            push_string(out, ParsedChunk::Thinking, &item[key]);
        }
    }
}

fn append_google_display(value: &Value, out: &mut Vec<ParsedChunk>) {
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
