use super::super::types_ollama::{ChatMessage, ChatRequest};
use super::PayloadStats;
use serde_json::Value;

pub(super) fn responses_payload_stats(
    messages: &[ChatMessage],
    target: Option<&crate::services::reasoning_continuity::contract::ContinuationTarget>,
) -> PayloadStats {
    let Ok((instructions, input)) =
        crate::services::codex_client::convert::convert_messages_with_tools_and_continuity(
            messages,
            &[],
            target,
        )
    else {
        return PayloadStats::default();
    };
    let mut stats = PayloadStats {
        items: input.len(),
        instructions_chars: char_count(&instructions),
        ..Default::default()
    };
    for item in input {
        match item["type"].as_str() {
            Some("reasoning") => {
                stats.reasoning_fields += 1;
                stats.reasoning_chars += json_chars(&item);
            }
            Some("function_call") => stats.tool_calls += 1,
            Some("function_call_output") => stats.tool_results += 1,
            _ if item["role"].as_str() == Some("assistant") => {
                stats.assistant_items += 1;
                stats.assistant_content_chars += value_text_chars(&item["content"]);
            }
            _ => {}
        }
    }
    stats
}

pub(super) fn chat_payload_stats(
    provider_id: &str,
    messages: &[ChatMessage],
    target: Option<&crate::services::reasoning_continuity::contract::ContinuationTarget>,
) -> PayloadStats {
    let converted = crate::services::llm::stream_convert::messages_to_openai(messages, provider_id);
    let mut payload = serde_json::json!({"messages": converted});
    if crate::services::llm::reasoning_wire::chat_text::apply_continuity(
        messages,
        target,
        &mut payload,
    )
    .is_err()
    {
        return PayloadStats::default();
    }
    let converted = payload["messages"].as_array().cloned().unwrap_or_default();
    let mut stats = PayloadStats {
        items: converted.len(),
        ..Default::default()
    };
    for item in converted {
        if item["role"].as_str() == Some("assistant") {
            stats.assistant_items += 1;
            if let Some(reasoning) = item
                .get("reasoning_content")
                .or_else(|| item.get("reasoning"))
            {
                stats.reasoning_fields += 1;
                stats.reasoning_chars += value_text_chars(reasoning);
            } else if let Some(details) = item.get("reasoning_details") {
                stats.reasoning_fields += 1;
                stats.reasoning_chars += json_chars(details);
            }
            if item["content"].is_null() {
                stats.assistant_content_nulls += 1;
            } else {
                stats.assistant_content_chars += value_text_chars(&item["content"]);
            }
            stats.tool_calls += item["tool_calls"].as_array().map_or(0, Vec::len);
        } else if item["role"].as_str() == Some("tool") {
            stats.tool_results += 1;
        }
    }
    stats
}

pub(super) fn ollama_payload_stats(request: &ChatRequest) -> PayloadStats {
    let Ok(payload) = super::super::ollama_wire::chat_request(request, &request.messages) else {
        return PayloadStats::default();
    };
    let Some(messages) = payload["messages"].as_array() else {
        return PayloadStats::default();
    };
    let mut stats = PayloadStats {
        items: messages.len(),
        ..Default::default()
    };
    for message in messages {
        if message["role"].as_str() == Some("assistant") {
            stats.assistant_items += 1;
            stats.assistant_content_chars += value_text_chars(&message["content"]);
            stats.tool_calls += message["tool_calls"].as_array().map_or(0, Vec::len);
            if message.get("thinking").is_some() {
                stats.reasoning_fields += 1;
                stats.reasoning_chars += value_text_chars(&message["thinking"]);
            }
        } else if message["role"].as_str() == Some("tool") {
            stats.tool_results += 1;
        }
    }
    stats
}

fn value_text_chars(value: &Value) -> usize {
    value.as_str().map_or(0, char_count)
}

fn json_chars(value: &Value) -> usize {
    serde_json::to_string(value).map_or(0, |value| char_count(&value))
}

fn char_count(value: &str) -> usize {
    value.chars().count()
}

#[cfg(test)]
#[path = "stream_diagnostics_payload_stats_tests.rs"]
mod tests;
