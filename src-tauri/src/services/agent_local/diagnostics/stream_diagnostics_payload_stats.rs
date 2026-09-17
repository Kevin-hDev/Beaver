use super::PayloadStats;
use serde_json::Value;

pub(super) fn payload_stats(kind: &str, payload: &Value) -> PayloadStats {
    let mut stats = PayloadStats {
        available_tools: payload["tools"].as_array().map_or(0, Vec::len),
        ..Default::default()
    };
    match kind {
        "responses" => responses(payload, &mut stats),
        "anthropic_messages" => anthropic(payload, &mut stats),
        "ollama_chat" => chat(payload, &mut stats, true),
        _ => chat(payload, &mut stats, false),
    }
    stats
}

fn responses(payload: &Value, stats: &mut PayloadStats) {
    stats.instructions_chars = text_chars(&payload["instructions"]);
    let Some(input) = payload["input"].as_array() else {
        return;
    };
    stats.items = input.len();
    for item in input {
        match item["type"].as_str() {
            Some("reasoning") => {
                stats.reasoning_fields += 1;
                stats.reasoning_chars += json_chars(item);
            }
            Some("function_call") => stats.tool_calls += 1,
            Some("function_call_output") => stats.tool_results += 1,
            _ if item["role"].as_str() == Some("assistant") => {
                stats.assistant_items += 1;
                stats.assistant_content_chars += text_chars(&item["content"]);
            }
            _ => {}
        }
    }
}

fn anthropic(payload: &Value, stats: &mut PayloadStats) {
    stats.instructions_chars = text_chars(&payload["system"]);
    let Some(messages) = payload["messages"].as_array() else {
        return;
    };
    stats.items = messages.len();
    for message in messages {
        let assistant = message["role"].as_str() == Some("assistant");
        if assistant {
            stats.assistant_items += 1;
        }
        for block in message["content"].as_array().into_iter().flatten() {
            match block["type"].as_str() {
                Some("thinking" | "redacted_thinking") if assistant => {
                    stats.reasoning_fields += 1;
                    stats.reasoning_chars += json_chars(block);
                }
                Some("text") if assistant => {
                    stats.assistant_content_chars += text_chars(&block["text"])
                }
                Some("tool_use") if assistant => stats.tool_calls += 1,
                Some("tool_result") => stats.tool_results += 1,
                _ => {}
            }
        }
    }
}

fn chat(payload: &Value, stats: &mut PayloadStats, ollama: bool) {
    let Some(messages) = payload["messages"].as_array() else {
        return;
    };
    stats.items = messages.len();
    for message in messages {
        match message["role"].as_str() {
            Some("system" | "developer") => {
                stats.instructions_chars += text_chars(&message["content"])
            }
            Some("assistant") => {
                stats.assistant_items += 1;
                if message["content"].is_null() {
                    stats.assistant_content_nulls += 1;
                } else {
                    stats.assistant_content_chars += text_chars(&message["content"]);
                }
                stats.tool_calls += message["tool_calls"].as_array().map_or(0, Vec::len);
                let reasoning = if ollama {
                    message.get("thinking")
                } else {
                    message
                        .get("reasoning_content")
                        .or_else(|| message.get("reasoning"))
                        .or_else(|| message.get("reasoning_details"))
                };
                if let Some(reasoning) = reasoning {
                    stats.reasoning_fields += 1;
                    stats.reasoning_chars += if reasoning.is_string() {
                        text_chars(reasoning)
                    } else {
                        json_chars(reasoning)
                    };
                }
            }
            Some("tool") => stats.tool_results += 1,
            _ => {}
        }
    }
}

fn text_chars(value: &Value) -> usize {
    match value {
        Value::String(text) => text.chars().count(),
        Value::Array(values) => values.iter().map(text_chars).sum(),
        Value::Object(map) => map
            .get("text")
            .or_else(|| map.get("content"))
            .map_or(0, text_chars),
        _ => 0,
    }
}

fn json_chars(value: &Value) -> usize {
    serde_json::to_string(value).map_or(0, |value| value.chars().count())
}

#[cfg(test)]
#[path = "stream_diagnostics_payload_stats_tests.rs"]
mod tests;
