use super::stream_diagnostics_support as support;
use super::types_ollama::{ChatMessage, ChatRequest};
use serde_json::Value;

#[derive(Debug, Default, PartialEq)]
struct PayloadStats {
    items: usize,
    assistant_items: usize,
    reasoning_fields: usize,
    reasoning_chars: usize,
    assistant_content_chars: usize,
    assistant_content_nulls: usize,
    tool_calls: usize,
    tool_results: usize,
    instructions_chars: usize,
}

pub async fn record_api_payload(
    session_id: &str,
    request_id: &str,
    turn: usize,
    provider_id: &str,
    messages: &[ChatMessage],
) {
    let (kind, stats) = if provider_id == "codex-oauth" {
        ("responses", codex_payload_stats(messages))
    } else {
        (
            "chat_completions",
            openai_payload_stats(provider_id, messages),
        )
    };
    record_payload(session_id, request_id, turn, provider_id, kind, stats).await;
}

pub async fn record_ollama_payload(
    session_id: &str,
    request_id: &str,
    turn: usize,
    request: &ChatRequest,
) {
    let mut stats = native_payload_stats(&request.messages);
    stats.tool_calls += request.tools.as_ref().map_or(0, Vec::len);
    record_payload(session_id, request_id, turn, "ollama", "ollama_chat", stats).await;
    record_ollama_tool_messages(session_id, request_id, turn, &request.messages).await;
}

/// Log ciblé sur les messages `role="tool"` du payload : taille du contenu,
/// tool_name, et présence/absence de tool_call_id.
async fn record_ollama_tool_messages(
    session_id: &str,
    request_id: &str,
    turn: usize,
    messages: &[ChatMessage],
) {
    for (i, m) in messages.iter().enumerate() {
        if m.role != "tool" {
            continue;
        }
        let content_chars = m.content.chars().count();
        let tool_name = m.tool_name.clone().unwrap_or_default();
        let has_id = m.tool_call_id.is_some();
        let message = format!(
            "ollama_tool_msg turn={} idx={} tool_name={} content_chars={} has_tool_call_id={}",
            turn + 1,
            i,
            tool_name,
            content_chars,
            has_id
        );
        let _ = support::update_run(session_id, request_id, |_session, run| {
            support::push_event(run, "ollama_tool_msg", &message, None, None);
        })
        .await;
    }
}

async fn record_payload(
    session_id: &str,
    request_id: &str,
    turn: usize,
    provider_id: &str,
    kind: &str,
    stats: PayloadStats,
) {
    let message = format!(
        "provider_payload provider={} kind={} turn={} items={} assistant={} reasoning_fields={} reasoning_chars={} assistant_content_chars={} content_nulls={} tool_calls={} tool_results={} instructions_chars={}",
        provider_id,
        kind,
        turn + 1,
        stats.items,
        stats.assistant_items,
        stats.reasoning_fields,
        stats.reasoning_chars,
        stats.assistant_content_chars,
        stats.assistant_content_nulls,
        stats.tool_calls,
        stats.tool_results,
        stats.instructions_chars
    );
    let _ = support::update_run(session_id, request_id, |_session, run| {
        run.phase = "provider_payload".to_string();
        run.safe_summary = Some(message.clone());
        support::push_event(run, "provider_payload", &message, None, None);
    })
    .await;
}

fn codex_payload_stats(messages: &[ChatMessage]) -> PayloadStats {
    let (instructions, input) = crate::services::codex_client::convert::convert_messages(messages);
    let mut stats = PayloadStats {
        items: input.len(),
        instructions_chars: char_count(&instructions),
        ..Default::default()
    };
    for item in input {
        if item["type"].as_str() == Some("function_call") {
            stats.tool_calls += 1;
        } else if item["type"].as_str() == Some("function_call_output") {
            stats.tool_results += 1;
        } else if item["role"].as_str() == Some("assistant") {
            stats.assistant_items += 1;
            stats.assistant_content_chars += value_text_chars(&item["content"]);
        }
    }
    stats
}

fn openai_payload_stats(provider_id: &str, messages: &[ChatMessage]) -> PayloadStats {
    let converted = crate::services::llm::stream_convert::messages_to_openai(messages, provider_id);
    let mut stats = PayloadStats {
        items: converted.len(),
        ..Default::default()
    };
    for item in converted {
        if item["role"].as_str() == Some("assistant") {
            stats.assistant_items += 1;
            if item.get("reasoning_content").is_some() {
                stats.reasoning_fields += 1;
                stats.reasoning_chars += value_text_chars(&item["reasoning_content"]);
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

fn native_payload_stats(messages: &[ChatMessage]) -> PayloadStats {
    let mut stats = PayloadStats {
        items: messages.len(),
        ..Default::default()
    };
    for message in messages {
        if message.role == "assistant" {
            stats.assistant_items += 1;
            stats.assistant_content_chars += char_count(&message.content);
            stats.tool_calls += message.tool_calls.as_ref().map_or(0, Vec::len);
            if let Some(reasoning) = message.reasoning_content.as_ref() {
                stats.reasoning_fields += 1;
                stats.reasoning_chars += char_count(reasoning);
            }
        } else if message.role == "tool" {
            stats.tool_results += 1;
        }
    }
    stats
}

fn value_text_chars(value: &Value) -> usize {
    value.as_str().map_or(0, char_count)
}

fn char_count(value: &str) -> usize {
    value.chars().count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::agent_local::types_ollama::{ToolCallFunction, ToolCallOllama};
    use serde_json::json;

    fn assistant_with_reasoning() -> ChatMessage {
        ChatMessage {
            role: "assistant".to_string(),
            content: "".to_string(),
            reasoning_content: Some("réflexion".to_string()),
            tool_calls: Some(vec![ToolCallOllama {
                id: Some("call_1".to_string()),
                extra_content: None,
                function: ToolCallFunction {
                    name: "grep".to_string(),
                    arguments: json!({"pattern": "x"}),
                },
            }]),
            ..Default::default()
        }
    }

    #[test]
    fn codex_payload_drops_reasoning_content() {
        let stats = codex_payload_stats(&[assistant_with_reasoning()]);
        assert_eq!(stats.reasoning_fields, 0);
        assert_eq!(stats.reasoning_chars, 0);
        assert_eq!(stats.assistant_items, 0);
        assert_eq!(stats.tool_calls, 1);
    }

    #[test]
    fn openai_payload_keeps_reasoning_content() {
        let stats = openai_payload_stats("zai", &[assistant_with_reasoning()]);
        assert_eq!(stats.reasoning_fields, 1);
        assert_eq!(stats.reasoning_chars, 9);
        assert_eq!(stats.assistant_content_nulls, 1);
        assert_eq!(stats.tool_calls, 1);
    }
}
