use crate::services::agent_local::types_ollama::ChatMessage;

pub fn estimate_tool_tokens(tools: &[serde_json::Value]) -> usize {
    tools.iter().fold(0usize, |total, tool| {
        total.saturating_add(crate::services::token_counting::estimate_text_tokens(
            &tool.to_string(),
        ))
    })
}

pub fn estimate_request_tokens(messages: &[ChatMessage], tools: &[serde_json::Value]) -> usize {
    super::token_estimate::estimate_tokens(messages).saturating_add(estimate_tool_tokens(tools))
}

pub fn estimate_request_tokens_for_provider(
    provider_id: &str,
    messages: &[ChatMessage],
    tools: &[serde_json::Value],
) -> usize {
    super::token_estimate::estimate_tokens_for_provider(provider_id, messages)
        .saturating_add(estimate_tool_tokens(tools))
}

pub fn estimate_textual_request_tokens_for_provider(
    provider_id: &str,
    messages: &[ChatMessage],
    tools: &[serde_json::Value],
) -> usize {
    super::token_estimate::estimate_textual_tokens_for_provider(provider_id, messages)
        .saturating_add(estimate_tool_tokens(tools))
}
