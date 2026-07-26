use crate::services::agent_local::types_ollama::ChatMessage;
use std::path::Path;

use super::context_capsules_disk_collect::{recent_disk_file_events, recent_tool_events};

pub(crate) const MAX_MANUAL_FILES: usize = 5;
pub(crate) const MAX_AUTO_FILES: usize = 15;
pub(crate) const MAX_RECENT_TOOLS: usize = 3;
pub(crate) const TOKENS_PER_CHAR: usize = 4;
pub(crate) const MIN_TOTAL_TOKENS: usize = 4_000;
pub(crate) const MAX_TOTAL_TOKENS: usize = 20_000;
pub(crate) const MAX_PER_FILE_TOKENS: usize = 8_000;
pub(crate) const MAX_PER_TOOL_TOKENS: usize = 4_000;
const TOTAL_PERCENT: u64 = 5;

#[derive(Debug, Clone, Copy)]
pub enum CompressionMode {
    Manual,
    Auto { request_start_index: usize },
}

pub(crate) struct CapsuleEvent {
    pub tool: String,
    pub path: String,
    pub result: String,
}

pub async fn compression_context_message(
    messages: &[ChatMessage],
    context_window: u64,
    working_dir: &Path,
    mode: CompressionMode,
) -> Option<ChatMessage> {
    let scan = scan_messages(messages, mode);
    let file_events = recent_disk_file_events(scan, working_dir, mode).await;
    let tool_events = recent_tool_events(scan);
    if file_events.is_empty() && tool_events.is_empty() {
        return None;
    }
    Some(build_message(file_events, tool_events, context_window))
}

fn scan_messages(messages: &[ChatMessage], mode: CompressionMode) -> &[ChatMessage] {
    match mode {
        CompressionMode::Manual => messages,
        CompressionMode::Auto {
            request_start_index,
        } => &messages[request_start_index.min(messages.len())..],
    }
}

fn build_message(
    file_events: Vec<CapsuleEvent>,
    tool_events: Vec<CapsuleEvent>,
    context_window: u64,
) -> ChatMessage {
    let total_tokens = capsule_total_tokens(context_window);
    let total_events = file_events.len() + tool_events.len();
    let per_file_chars = (total_tokens / total_events.max(1))
        .min(MAX_PER_FILE_TOKENS)
        .saturating_mul(TOKENS_PER_CHAR);
    let per_tool_chars = (total_tokens / total_events.max(1))
        .min(MAX_PER_TOOL_TOKENS)
        .saturating_mul(TOKENS_PER_CHAR);
    let mut content = String::from("Recent file context preserved across compression:\n");
    for event in file_events {
        content.push_str(&format!(
            "\n- {}: {}\n{}\n",
            event.tool,
            event.path,
            truncate_chars(&event.result, per_file_chars)
        ));
    }
    if !tool_events.is_empty() {
        content.push_str("\nRecent tool context preserved across compression:\n");
        for event in tool_events {
            content.push_str(&format!(
                "\n- {}\n{}\n",
                event.tool,
                truncate_chars(&event.result, per_tool_chars)
            ));
        }
    }
    ChatMessage {
        role: "user".to_string(),
        content,
        ..Default::default()
    }
}

fn capsule_total_tokens(context_window: u64) -> usize {
    if context_window == 0 {
        return MIN_TOTAL_TOKENS;
    }
    let target = (context_window.saturating_mul(TOTAL_PERCENT) / 100) as usize;
    target.clamp(MIN_TOTAL_TOKENS, MAX_TOTAL_TOKENS)
}

fn truncate_chars(input: &str, max_chars: usize) -> String {
    if input.chars().count() <= max_chars {
        return input.to_string();
    }
    let kept: String = input.chars().take(max_chars).collect();
    format!("{kept}\n[content truncated for context budget]")
}

#[cfg(test)]
#[path = "context_capsules_disk_tests.rs"]
mod tests;
