use crate::services::agent_local::types_ollama::ChatMessage;

pub fn tool_chain_is_closed(messages: &[ChatMessage]) -> bool {
    let mut pending = 0usize;
    for message in messages.iter().filter(|message| message.role != "system") {
        if pending > 0 && message.role != "tool" {
            return false;
        }
        if message.role == "assistant" {
            pending = message.tool_calls.as_ref().map_or(0, Vec::len);
        } else if message.role == "tool" && pending > 0 {
            pending -= 1;
        }
    }
    pending == 0
}

pub fn include_chat_message(message: &ChatMessage) -> bool {
    message.role != "system" && !is_compress_command(&message.content)
}

fn is_compress_command(content: &str) -> bool {
    content.trim() == "/compress"
}
