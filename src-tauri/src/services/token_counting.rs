use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::agent_local::types_session::AgentMessage;

const ASCII_UNITS: usize = 1;
const NON_ASCII_UNITS: usize = 2;
const WIDE_UNITS: usize = 5;
const UNITS_PER_TOKEN: usize = 4;

pub fn estimate_chat_tokens(messages: &[ChatMessage]) -> usize {
    messages.iter().map(estimate_chat_message_tokens).sum()
}

pub fn estimate_chat_tokens_without_reasoning(messages: &[ChatMessage]) -> usize {
    messages
        .iter()
        .map(|message| estimate_chat_message_tokens_with_reasoning(message, false))
        .sum()
}

pub fn estimate_chat_message_tokens_without_reasoning(message: &ChatMessage) -> usize {
    estimate_chat_message_tokens_with_reasoning(message, false)
}

pub fn estimate_text_tokens(input: &str) -> usize {
    token_count_from_units(text_units(input))
}

pub fn estimate_agent_messages_tokens(messages: &[AgentMessage]) -> u32 {
    messages
        .iter()
        .map(estimate_agent_message_tokens)
        .sum::<usize>()
        .min(u32::MAX as usize) as u32
}

pub fn estimate_chat_message_tokens(message: &ChatMessage) -> usize {
    estimate_chat_message_tokens_with_reasoning(message, true)
}

fn estimate_chat_message_tokens_with_reasoning(
    message: &ChatMessage,
    include_reasoning: bool,
) -> usize {
    let mut units = text_units(&message.content);
    if include_reasoning {
        units += message
            .reasoning_content
            .as_deref()
            .map(text_units)
            .unwrap_or(0);
    }
    if let Some(calls) = &message.tool_calls {
        for call in calls {
            units += text_units(&call.function.name);
            units += text_units(&call.function.arguments.to_string());
        }
    }
    token_count_from_units(units) + image_tokens(message.images.as_ref().map(Vec::len).unwrap_or(0))
}

pub fn estimate_agent_message_tokens(message: &AgentMessage) -> usize {
    let mut units = text_units(&message.content);
    units += message.thinking.as_deref().map(text_units).unwrap_or(0);
    if let Some(calls) = &message.tool_calls {
        for call in calls {
            units += text_units(&call.function.name);
            units += text_units(&call.function.arguments.to_string());
        }
    }
    if let Some(activities) = &message.tool_activities {
        for activity in activities {
            units += text_units(&activity.name);
            units += text_units(&restored_tool_arguments(activity));
            units += activity.result.as_deref().map(text_units).unwrap_or(0);
        }
    }
    token_count_from_units(units)
}

fn restored_tool_arguments(
    activity: &crate::services::agent_local::types_session::ToolActivityRecord,
) -> String {
    if let Some(arguments) = &activity.args {
        return arguments.to_string();
    }
    let key = match activity.name.as_str() {
        "web_search" => "query",
        "web_fetch" => "url",
        "bash" => "command",
        "grep" | "glob" => "pattern",
        "transform_image" => "input_path",
        "read_file" | "write_file" | "edit_file" | "list_dir" | "read_spreadsheet"
        | "read_document" | "write_spreadsheet" | "write_document" => "path",
        _ => "input",
    };
    serde_json::json!({ key: activity.summary }).to_string()
}

pub fn sum_real_counts(left: Option<u32>, right: Option<u32>) -> Option<u32> {
    Some(left?.saturating_add(right?))
}

pub fn add_real_count(total: &mut Option<u32>, value: Option<u32>) {
    *total = sum_real_counts(*total, value);
}

pub fn text_units(input: &str) -> usize {
    input.chars().map(char_units).sum()
}

pub fn token_count_from_units(units: usize) -> usize {
    units.div_ceil(UNITS_PER_TOKEN)
}

pub fn max_text_units(tokens: usize) -> usize {
    tokens.saturating_mul(UNITS_PER_TOKEN)
}

fn image_tokens(count: usize) -> usize {
    count * crate::services::llm::vision::IMAGE_TOKEN_ESTIMATE
}

fn char_units(ch: char) -> usize {
    if ch.is_ascii() {
        ASCII_UNITS
    } else if is_wide_or_emoji(ch) {
        WIDE_UNITS
    } else {
        NON_ASCII_UNITS
    }
}

fn is_wide_or_emoji(ch: char) -> bool {
    matches!(
        ch as u32,
        0x1100..=0x11FF
            | 0x2E80..=0x2EFF
            | 0x2F00..=0x2FDF
            | 0x3000..=0x30FF
            | 0x3130..=0x318F
            | 0x31A0..=0x31BF
            | 0x31F0..=0x31FF
            | 0x3400..=0x4DBF
            | 0x4E00..=0x9FFF
            | 0xAC00..=0xD7AF
            | 0xF900..=0xFAFF
            | 0xFE00..=0xFE0F
            | 0xFF00..=0xFFEF
            | 0x1F000..=0x1FAFF
            | 0x20000..=0x2CEAF
    )
}

#[cfg(test)]
#[path = "token_counting_tests.rs"]
mod tests;
