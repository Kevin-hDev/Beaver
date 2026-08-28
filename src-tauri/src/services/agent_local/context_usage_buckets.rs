use super::types_ollama::ChatMessage;
use crate::services::token_counting;
use serde::Serialize;

const MESSAGES: usize = 0;
const SYSTEM_TOOLS: usize = 1;
const MCP_CONNECTORS: usize = 2;
const META_CONTEXT: usize = 3;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ContextUsageSeed {
    pub meta_context_tokens: usize,
    pub skill_context_tokens: usize,
    pub memory_context_tokens: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestContextUsage {
    pub messages: u32,
    pub system_tools: u32,
    pub mcp_connectors: u32,
    pub skills: u32,
    pub memory: u32,
    pub meta_context: u32,
    pub system_prompt: u32,
    pub reasoning_included: bool,
}

impl RequestContextUsage {
    pub fn from_request(
        provider_id: &str,
        messages: &[ChatMessage],
        tools: &[serde_json::Value],
        seed: ContextUsageSeed,
    ) -> Self {
        let reasoning_included = provider_id != crate::services::codex_client::PROVIDER_ID;
        let mut classified = [0usize; 4];
        let mut system_tokens = 0usize;

        for message in messages {
            if message.role == "system" {
                system_tokens =
                    system_tokens.saturating_add(message_tokens(message, reasoning_included));
            } else {
                add_message(&mut classified, message, reasoning_included);
            }
        }
        for definition in tools {
            let tokens = token_counting::estimate_text_tokens(&definition.to_string());
            let bucket = definition
                .pointer("/function/name")
                .and_then(serde_json::Value::as_str)
                .map(tool_bucket)
                .unwrap_or(SYSTEM_TOOLS);
            classified[bucket] = classified[bucket].saturating_add(tokens);
        }

        let (skills, memory, meta_context, system_prompt) =
            split_system_tokens(system_tokens, seed);
        Self {
            messages: bounded(classified[MESSAGES]),
            system_tools: bounded(classified[SYSTEM_TOOLS]),
            mcp_connectors: bounded(classified[MCP_CONNECTORS]),
            skills: bounded(skills),
            memory: bounded(memory),
            meta_context: bounded(meta_context.saturating_add(classified[META_CONTEXT])),
            system_prompt: bounded(system_prompt),
            reasoning_included,
        }
    }
}

fn add_message(target: &mut [usize; 4], message: &ChatMessage, include_reasoning: bool) {
    let mut units = [0usize; 4];
    let content_bucket = message
        .tool_name
        .as_deref()
        .map(tool_bucket)
        .unwrap_or(MESSAGES);
    units[content_bucket] = token_counting::text_units(&message.content);
    if include_reasoning {
        units[MESSAGES] = units[MESSAGES].saturating_add(
            message
                .tool_loop_reasoning
                .as_deref()
                .map(token_counting::text_units)
                .unwrap_or(0),
        );
    }
    for call in message.tool_calls.iter().flatten() {
        let bucket = tool_bucket(&call.function.name);
        units[bucket] = units[bucket]
            .saturating_add(token_counting::text_units(&call.function.name))
            .saturating_add(token_counting::text_units(
                &call.function.arguments.to_string(),
            ));
    }

    let allocated = allocate_text_tokens(units);
    for index in 0..target.len() {
        target[index] = target[index].saturating_add(allocated[index]);
    }
    let image_count = message.images.as_ref().map(Vec::len).unwrap_or(0);
    target[MESSAGES] = target[MESSAGES].saturating_add(
        image_count.saturating_mul(crate::services::llm::vision::IMAGE_TOKEN_ESTIMATE),
    );
}

fn allocate_text_tokens(units: [usize; 4]) -> [usize; 4] {
    let total_units = units
        .iter()
        .fold(0usize, |sum, value| sum.saturating_add(*value));
    if total_units == 0 {
        return [0; 4];
    }
    let total_tokens = token_counting::token_count_from_units(total_units);
    let mut allocated = [0usize; 4];
    let mut remainders = [0u128; 4];
    let denominator = total_units as u128;
    for index in 0..units.len() {
        let weighted = (units[index] as u128).saturating_mul(total_tokens as u128);
        allocated[index] = (weighted / denominator) as usize;
        remainders[index] = weighted % denominator;
    }
    let mut missing = total_tokens.saturating_sub(allocated.iter().sum());
    while missing > 0 {
        let index = remainders
            .iter()
            .enumerate()
            .max_by_key(|(_, remainder)| **remainder)
            .map(|(index, _)| index)
            .unwrap_or(MESSAGES);
        allocated[index] = allocated[index].saturating_add(1);
        remainders[index] = 0;
        missing -= 1;
    }
    allocated
}

fn message_tokens(message: &ChatMessage, include_reasoning: bool) -> usize {
    let mut units = token_counting::text_units(&message.content);
    if include_reasoning {
        units = units.saturating_add(
            message
                .tool_loop_reasoning
                .as_deref()
                .map(token_counting::text_units)
                .unwrap_or(0),
        );
    }
    for call in message.tool_calls.iter().flatten() {
        units = units
            .saturating_add(token_counting::text_units(&call.function.name))
            .saturating_add(token_counting::text_units(
                &call.function.arguments.to_string(),
            ));
    }
    let images = message.images.as_ref().map(Vec::len).unwrap_or(0);
    token_counting::token_count_from_units(units)
        .saturating_add(images.saturating_mul(crate::services::llm::vision::IMAGE_TOKEN_ESTIMATE))
}

fn split_system_tokens(total: usize, seed: ContextUsageSeed) -> (usize, usize, usize, usize) {
    let mut remaining = total;
    let skills = take(&mut remaining, seed.skill_context_tokens);
    let memory = take(&mut remaining, seed.memory_context_tokens);
    let meta = take(&mut remaining, seed.meta_context_tokens);
    (skills, memory, meta, remaining)
}

fn take(remaining: &mut usize, requested: usize) -> usize {
    let value = requested.min(*remaining);
    *remaining -= value;
    value
}

fn tool_bucket(name: &str) -> usize {
    if name == "load_skill" {
        META_CONTEXT
    } else if is_mcp_tool(name) {
        MCP_CONNECTORS
    } else {
        SYSTEM_TOOLS
    }
}

fn is_mcp_tool(name: &str) -> bool {
    matches!(name, "mcp" | "mcp_tool" | "search_mcp_tools") || name.starts_with("mcp_")
}

fn bounded(value: usize) -> u32 {
    value.min(u32::MAX as usize) as u32
}

#[cfg(test)]
#[path = "context_usage_buckets_tests.rs"]
mod tests;
