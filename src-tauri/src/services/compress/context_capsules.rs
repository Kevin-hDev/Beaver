use crate::services::agent_local::types_ollama::ChatMessage;

const MAX_RECENT_FILES: usize = 3;
const MAX_RECENT_TOOLS: usize = 3;
const TOKENS_PER_CHAR: usize = 4;
const MIN_TOTAL_TOKENS: usize = 4_000;
const MAX_TOTAL_TOKENS: usize = 20_000;
const MAX_PER_FILE_TOKENS: usize = 8_000;
const MAX_PER_TOOL_TOKENS: usize = 4_000;
const TOTAL_PERCENT: u64 = 5;

struct FileEvent {
    tool: String,
    path: String,
    result: String,
}

pub fn recent_file_context_message(
    messages: &[ChatMessage],
    context_window: u64,
) -> Option<ChatMessage> {
    let file_events = recent_file_events(messages);
    let tool_events = recent_tool_events(messages);
    if file_events.is_empty() && tool_events.is_empty() {
        return None;
    }
    let total_tokens = capsule_total_tokens(context_window);
    let total_events = file_events.len() + tool_events.len();
    let per_file_tokens = (total_tokens / total_events.max(1)).min(MAX_PER_FILE_TOKENS);
    let per_tool_tokens = (total_tokens / total_events.max(1)).min(MAX_PER_TOOL_TOKENS);
    let per_file_chars = per_file_tokens.saturating_mul(TOKENS_PER_CHAR);
    let per_tool_chars = per_tool_tokens.saturating_mul(TOKENS_PER_CHAR);
    let mut content = String::from("Recent file context preserved across compression:\n");
    for event in file_events {
        content.push_str("\n- ");
        content.push_str(&event.tool);
        content.push_str(": ");
        content.push_str(&event.path);
        content.push('\n');
        content.push_str(&truncate_chars(&event.result, per_file_chars));
        content.push('\n');
    }
    if !tool_events.is_empty() {
        content.push_str("\nRecent tool context preserved across compression:\n");
        for event in tool_events {
            content.push_str("\n- ");
            content.push_str(&event.tool);
            content.push('\n');
            content.push_str(&truncate_chars(&event.result, per_tool_chars));
            content.push('\n');
        }
    }
    Some(ChatMessage {
        role: "user".to_string(),
        content,
        images: None,
        tool_calls: None,
        tool_name: None,
        tool_call_id: None,
        reasoning_content: None,
    })
}

pub fn insert_after_system(messages: &mut Vec<ChatMessage>, context: Option<ChatMessage>) {
    let Some(context) = context else {
        return;
    };
    let pos = messages
        .iter()
        .position(|m| m.role != "system")
        .unwrap_or(messages.len());
    messages.insert(pos, context);
}

fn capsule_total_tokens(context_window: u64) -> usize {
    if context_window == 0 {
        return MIN_TOTAL_TOKENS;
    }
    let target = (context_window.saturating_mul(TOTAL_PERCENT) / 100) as usize;
    target.clamp(MIN_TOTAL_TOKENS, MAX_TOTAL_TOKENS)
}

fn recent_file_events(messages: &[ChatMessage]) -> Vec<FileEvent> {
    let mut found = Vec::new();
    let mut pending: Vec<(String, String)> = Vec::new();
    for msg in messages {
        if msg.role == "assistant" {
            pending.clear();
            if let Some(calls) = &msg.tool_calls {
                for call in calls {
                    let name = call.function.name.clone();
                    if let Some(path) = file_path_for_tool(&name, &call.function.arguments) {
                        pending.push((name, path));
                    }
                }
            }
        } else if msg.role == "tool" {
            let next = pending.first().cloned().or_else(|| {
                msg.tool_name
                    .as_ref()
                    .and_then(|name| file_path_from_content(name, &msg.content))
            });
            if let Some((tool, path)) = next {
                found.push(FileEvent {
                    tool,
                    path,
                    result: msg.content.clone(),
                });
                if !pending.is_empty() {
                    pending.remove(0);
                }
            }
        }
    }
    let keep_from = found.len().saturating_sub(MAX_RECENT_FILES);
    found.into_iter().skip(keep_from).collect()
}

fn recent_tool_events(messages: &[ChatMessage]) -> Vec<FileEvent> {
    let mut found = Vec::new();
    for msg in messages {
        if msg.role != "tool" {
            continue;
        }
        let Some(tool) = msg.tool_name.as_deref() else {
            continue;
        };
        if !is_context_tool(tool) || file_path_from_content(tool, &msg.content).is_some() {
            continue;
        }
        found.push(FileEvent {
            tool: tool.to_string(),
            path: String::new(),
            result: msg.content.clone(),
        });
    }
    let keep_from = found.len().saturating_sub(MAX_RECENT_TOOLS);
    found.into_iter().skip(keep_from).collect()
}

fn file_path_for_tool(tool: &str, args: &serde_json::Value) -> Option<String> {
    if !matches!(
        tool,
        "read_file"
            | "write_file"
            | "edit_file"
            | "read_document"
            | "write_document"
            | "read_spreadsheet"
            | "write_spreadsheet"
    ) {
        return None;
    }
    args.get("path")?.as_str().map(ToString::to_string)
}

fn file_path_from_content(tool: &str, content: &str) -> Option<(String, String)> {
    if !matches!(tool, "write_file" | "edit_file") {
        return None;
    }
    content
        .split_once(':')
        .map(|(_, path)| (tool.to_string(), path.trim().to_string()))
}

fn is_context_tool(tool: &str) -> bool {
    matches!(
        tool,
        "bash" | "grep" | "glob" | "list_dir" | "web_fetch" | "web_search" | "mcp_tool" | "mcp"
    ) || tool.starts_with("mcp_")
}

fn truncate_chars(input: &str, max_chars: usize) -> String {
    if input.chars().count() <= max_chars {
        return input.to_string();
    }
    let kept: String = input.chars().take(max_chars).collect();
    format!("{kept}\n[content truncated for context budget]")
}
