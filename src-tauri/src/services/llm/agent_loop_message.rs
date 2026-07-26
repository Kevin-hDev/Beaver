use crate::services::agent_local::types_ollama::{
    ChatMessage, StreamResult, ToolCallFunction, ToolCallOllama,
};

pub fn build_assistant_message(result: &StreamResult) -> ChatMessage {
    let tool_calls = if result.tool_calls.is_empty() {
        None
    } else {
        Some(
            result
                .tool_calls
                .iter()
                .enumerate()
                .map(|(i, (name, args))| ToolCallOllama {
                    id: result.tool_call_ids.get(i).cloned(),
                    extra_content: result.tool_call_extra_content.get(i).cloned().flatten(),
                    function: ToolCallFunction {
                        name: name.clone(),
                        arguments: args.clone(),
                    },
                })
                .collect(),
        )
    };
    let reasoning = if result.thinking.is_empty() {
        None
    } else {
        Some(result.thinking.clone())
    };
    ChatMessage {
        role: "assistant".to_string(),
        content: result.content.clone(),
        tool_calls,
        reasoning_content: reasoning,
        ..Default::default()
    }
}

pub fn build_for_plan(result: &StreamResult, plan_active: bool) -> ChatMessage {
    let mut message = build_assistant_message(result);
    if plan_active && !result.tool_calls.is_empty() {
        message.content.clear();
    }
    message
}
