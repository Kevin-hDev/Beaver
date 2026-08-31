use crate::services::agent_local::types_session::AgentMessage;
use std::collections::HashSet;

pub const MAX_TOOL_EVENTS: usize = 100;

pub fn closed_chain_end(
    messages: &[AgentMessage],
    assistant_index: usize,
    turn_end: usize,
) -> Result<usize, &'static str> {
    let calls = messages[assistant_index]
        .tool_calls
        .as_deref()
        .filter(|calls| !calls.is_empty())
        .ok_or("compression_checkpoint_invalid_tool_chain")?;
    let end = assistant_index
        .saturating_add(calls.len())
        .saturating_add(1);
    let results = messages
        .get(assistant_index + 1..end)
        .filter(|_| end <= turn_end)
        .ok_or("compression_checkpoint_invalid_tool_chain")?;
    let matches = calls.iter().zip(results).all(|(call, result)| {
        result.role == "tool"
            && result.tool_call_id.as_deref() == Some(call.id.as_str())
            && result.tool_name.as_deref() == Some(call.function.name.as_str())
    });
    let unique_ids = calls
        .iter()
        .map(|call| call.id.as_str())
        .collect::<HashSet<_>>()
        .len()
        == calls.len();
    (matches && unique_ids)
        .then_some(end)
        .ok_or("compression_checkpoint_invalid_tool_chain")
}

pub fn validate_active_turn(turn: &[AgentMessage]) -> Result<(), &'static str> {
    if turn.first().is_none_or(|message| message.role != "user") {
        return Err("compression_checkpoint_invalid_tool_chain");
    }
    let mut index = 1usize;
    while index < turn.len() {
        let message = &turn[index];
        if message.role != "assistant"
            || message
                .tool_calls
                .as_ref()
                .is_none_or(|calls| calls.is_empty())
        {
            return Err("compression_checkpoint_invalid_tool_chain");
        }
        index = closed_chain_end(turn, index, turn.len())?;
    }
    Ok(())
}

pub fn excerpt_result(message: &AgentMessage, max_tokens: u32) -> AgentMessage {
    if super::token_estimate::estimate_checkpoint_message_tokens(message) <= max_tokens {
        return message.clone();
    }
    let mut excerpt = message.clone();
    let reference =
        crate::services::agent_local::tool_result_truncate::full_result_reference(&message.content)
            .unwrap_or_default();
    excerpt.content = super::checkpoint_messages::bounded_excerpt(
        &message.content,
        max_tokens,
        "\n[tool result excerpt]\n",
        reference,
    );
    excerpt
}
