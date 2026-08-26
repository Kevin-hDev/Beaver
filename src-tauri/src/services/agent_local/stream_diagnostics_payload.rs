use super::stream_diagnostics_support as support;
use super::types_ollama::{ChatMessage, ChatRequest};

#[path = "stream_diagnostics_payload_stats.rs"]
mod payload_stats;
use payload_stats::{chat_payload_stats, ollama_payload_stats, responses_payload_stats};

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
    continuation_target: Option<
        &crate::services::reasoning_continuity::contract::ContinuationTarget,
    >,
) {
    let (kind, stats) = if matches!(provider_id, "openai" | "codex-oauth" | "xai-oauth") {
        (
            "responses",
            responses_payload_stats(messages, continuation_target),
        )
    } else {
        (
            "chat_completions",
            chat_payload_stats(provider_id, messages, continuation_target),
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
    let mut stats = ollama_payload_stats(request);
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
