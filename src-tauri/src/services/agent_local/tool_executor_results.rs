use super::stream_events::AgentEventEmitter;
use super::types_ollama::{ChatMessage, StreamEvent};
use super::types_tools::ToolResult;

pub fn push_tool_result(
    on_event: &AgentEventEmitter,
    messages: &mut Vec<ChatMessage>,
    name: &str,
    tr: ToolResult,
    tool_call_index: usize,
    tool_call_id: Option<&str>,
    resolved_path: Option<String>,
) {
    emit_tool_result(on_event, name, &tr, tool_call_index, resolved_path);
    push_tool_message(messages, name, tr, tool_call_id);
}

pub fn emit_tool_result(
    on_event: &AgentEventEmitter,
    name: &str,
    tr: &ToolResult,
    tool_call_index: usize,
    resolved_path: Option<String>,
) {
    let domain = super::memory_tool::resolved_path_domain(resolved_path.as_deref());
    let _ = on_event.send(StreamEvent::ToolResult {
        name: name.to_string(),
        content: tr.content.clone(),
        is_error: tr.is_error,
        truncated: tr.truncated,
        display_summary: tr.display_summary.clone(),
        tool_call_index,
        resolved_path,
        domain,
        affected_paths: tr.affected_paths.clone(),
        file_changes: tr.file_changes.clone(),
        start_line: tr.start_line,
    });
}

pub fn push_tool_message(
    messages: &mut Vec<ChatMessage>,
    name: &str,
    tr: ToolResult,
    tool_call_id: Option<&str>,
) {
    messages.push(ChatMessage {
        role: "tool".to_string(),
        content: tr.content,
        images: None,
        tool_calls: None,
        tool_name: Some(name.to_string()),
        tool_call_id: tool_call_id.map(str::to_string),
        reasoning_content: None,
    });
}
