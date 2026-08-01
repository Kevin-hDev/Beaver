use super::stream_events::AgentEventEmitter;
use super::types_ollama::{ChatMessage, StreamEvent};
use super::types_tools::{ToolFollowUp, ToolResult};

pub fn push_tool_result(
    on_event: &AgentEventEmitter,
    messages: &mut Vec<ChatMessage>,
    name: &str,
    tr: ToolResult,
    tool_call_index: usize,
    tool_call_id: Option<&str>,
    resolved_path: Option<String>,
) -> ToolFollowUp {
    emit_tool_result(
        on_event,
        name,
        &tr,
        tool_call_index,
        tool_call_id,
        resolved_path,
    );
    push_tool_message(messages, name, tr, tool_call_id)
}

pub fn emit_tool_result(
    on_event: &AgentEventEmitter,
    name: &str,
    tr: &ToolResult,
    tool_call_index: usize,
    tool_call_id: Option<&str>,
    resolved_path: Option<String>,
) {
    let domain = super::memory_tool::resolved_path_domain(resolved_path.as_deref());
    let _ = on_event.send(StreamEvent::ToolResult {
        name: name.to_string(),
        content: tr.content.clone(),
        is_error: tr.is_error,
        status: tr.status,
        error: tr.error.clone(),
        warnings: tr.warnings.clone(),
        truncated: tr.truncated,
        display_summary: tr.display_summary().map(str::to_owned),
        tool_call_index,
        tool_call_id: tool_call_id.map(str::to_owned),
        resolved_path,
        domain,
        affected_paths: tr.affected_paths().to_vec(),
        file_changes: tr.file_changes().to_vec(),
        start_line: tr.start_line(),
    });
}

pub fn push_tool_message(
    messages: &mut Vec<ChatMessage>,
    name: &str,
    mut tr: ToolResult,
    tool_call_id: Option<&str>,
) -> ToolFollowUp {
    let follow_up = tr.take_follow_up();
    let content = super::tool_result_model::render(name, &tr);
    messages.push(ChatMessage {
        role: "tool".to_string(),
        content,
        images: None,
        tool_calls: None,
        tool_name: Some(name.to_string()),
        tool_call_id: tool_call_id.map(str::to_string),
        reasoning_content: None,
    });
    follow_up
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_receipt_stays_before_the_real_user_message() {
        let mut messages = Vec::new();
        let follow_up = push_tool_message(
            &mut messages,
            "ask_user_choice",
            ToolResult::ok("Interactive answer received.").with_user_message("User answer"),
            Some("call-1"),
        );

        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].role, "tool");
        assert_eq!(messages[0].content, "Interactive answer received.");
        assert_eq!(messages[0].tool_call_id.as_deref(), Some("call-1"));
        assert_eq!(
            follow_up,
            ToolFollowUp::UserMessage("User answer".into())
        );
    }

    #[test]
    fn model_message_keeps_error_status_and_code() {
        let mut messages = Vec::new();
        let _ = push_tool_message(
            &mut messages,
            "bash",
            ToolResult::error(
                "done",
                "shell_exit_nonzero",
                super::super::tool_result_contract::ToolErrorCategory::Execution,
                false,
            ),
            Some("call-1"),
        );

        let (metadata, output) = messages[0]
            .content
            .split_once('\n')
            .expect("structured result and raw output");
        let value: serde_json::Value = serde_json::from_str(metadata).expect("metadata");
        assert_eq!(value["status"], "error");
        assert_eq!(value["error"]["code"], "shell_exit_nonzero");
        assert_eq!(output, "done");
    }
}
