use crate::services::agent_local::tool_files;
use crate::services::agent_local::types_ollama::ChatMessage;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use super::context_capsules_disk::CapsuleEvent;
#[cfg(test)]
use super::context_capsules_disk::MAX_RECENT_TOOLS;

const UNAVAILABLE_MARKER: &str = "[file unavailable: deleted, binary, or unreadable]";

struct FileCandidate {
    tool: String,
    path: String,
}

pub async fn recent_disk_file_events(
    messages: &[ChatMessage],
    working_dir: &Path,
    max_files: usize,
) -> Vec<CapsuleEvent> {
    recent_disk_file_events_inner(messages, working_dir, max_files, None).await
}

pub async fn recent_disk_file_events_bounded(
    messages: &[ChatMessage],
    working_dir: &Path,
    max_files: usize,
    max_tokens_per_file: u32,
) -> Vec<CapsuleEvent> {
    recent_disk_file_events_inner(messages, working_dir, max_files, Some(max_tokens_per_file)).await
}

async fn recent_disk_file_events_inner(
    messages: &[ChatMessage],
    working_dir: &Path,
    max_files: usize,
    max_tokens_per_file: Option<u32>,
) -> Vec<CapsuleEvent> {
    let mut seen = HashSet::<PathBuf>::new();
    let mut events = Vec::new();
    for candidate in file_candidates(messages).into_iter().rev() {
        let Some((resolved, result)) =
            read_current_state(&candidate.path, working_dir, max_tokens_per_file).await
        else {
            continue;
        };
        if !seen.insert(resolved) {
            continue;
        }
        events.push(CapsuleEvent {
            tool: candidate.tool,
            path: candidate.path,
            result,
        });
        if events.len() >= max_files {
            break;
        }
    }
    events.reverse();
    events
}

#[cfg(test)]
pub fn recent_tool_events(messages: &[ChatMessage]) -> Vec<CapsuleEvent> {
    let mut found = Vec::new();
    for msg in messages.iter().filter(|message| message.role == "tool") {
        let Some(tool) = msg.tool_name.as_deref() else {
            continue;
        };
        if !is_context_tool(tool) || file_path_from_content(tool, &msg.content).is_some() {
            continue;
        }
        found.push(CapsuleEvent {
            tool: tool.to_string(),
            path: String::new(),
            result: msg.content.clone(),
        });
    }
    let keep_from = found.len().saturating_sub(MAX_RECENT_TOOLS);
    found.into_iter().skip(keep_from).collect()
}

async fn read_current_state(
    path: &str,
    working_dir: &Path,
    max_tokens: Option<u32>,
) -> Option<(PathBuf, String)> {
    let resolved = tool_files::resolve_read_path(path, working_dir).ok()?;
    let content = match max_tokens {
        Some(tokens) => read_bounded_text(&resolved, tokens).await,
        None => tokio::fs::read_to_string(&resolved)
            .await
            .unwrap_or_else(|_| UNAVAILABLE_MARKER.to_string()),
    };
    Some((resolved, content))
}

async fn read_bounded_text(path: &Path, max_tokens: u32) -> String {
    use tokio::io::AsyncReadExt;

    let max_bytes = usize::try_from(max_tokens)
        .unwrap_or(usize::MAX)
        .saturating_mul(4)
        .max(1);
    let Ok(file) = tokio::fs::File::open(path).await else {
        return UNAVAILABLE_MARKER.to_string();
    };
    let mut bytes = Vec::with_capacity(max_bytes.min(64 * 1024));
    if file
        .take(max_bytes.saturating_add(1) as u64)
        .read_to_end(&mut bytes)
        .await
        .is_err()
    {
        return UNAVAILABLE_MARKER.to_string();
    }
    let truncated = bytes.len() > max_bytes;
    bytes.truncate(max_bytes);
    let text = match String::from_utf8(bytes) {
        Ok(text) => text,
        Err(error) if truncated && error.utf8_error().error_len().is_none() => {
            let valid = error.utf8_error().valid_up_to();
            let mut bytes = error.into_bytes();
            bytes.truncate(valid);
            String::from_utf8(bytes).unwrap_or_default()
        }
        Err(_) => return UNAVAILABLE_MARKER.to_string(),
    };
    if truncated {
        super::checkpoint_messages::bounded_excerpt(
            &text,
            max_tokens,
            "\n[file content truncated]",
            "",
        )
    } else {
        text
    }
}

fn file_candidates(messages: &[ChatMessage]) -> Vec<FileCandidate> {
    let mut found = Vec::new();
    let mut pending: Vec<(String, String)> = Vec::new();
    for msg in messages {
        if msg.role == "assistant" {
            pending.clear();
            if let Some(calls) = &msg.tool_calls {
                pending.extend(calls.iter().filter_map(|call| {
                    let name = call.function.name.clone();
                    file_path_for_tool(&name, &call.function.arguments).map(|path| (name, path))
                }));
            }
        } else if msg.role == "tool" {
            let next = pending.first().cloned().or_else(|| {
                msg.tool_name
                    .as_ref()
                    .and_then(|name| file_path_from_content(name, &msg.content))
            });
            if let Some((tool, path)) = next {
                found.push(FileCandidate { tool, path });
                if !pending.is_empty() {
                    pending.remove(0);
                }
            }
        }
    }
    found
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
    matches!(tool, "write_file" | "edit_file")
        .then(|| content.split_once(':'))
        .flatten()
        .map(|(_, path)| (tool.to_string(), path.trim().to_string()))
}

#[cfg(test)]
fn is_context_tool(tool: &str) -> bool {
    matches!(
        tool,
        "bash" | "grep" | "glob" | "list_dir" | "web_fetch" | "web_search" | "mcp_tool" | "mcp"
    ) || tool.starts_with("mcp_")
}
