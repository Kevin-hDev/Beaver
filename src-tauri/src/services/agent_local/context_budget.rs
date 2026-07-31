use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::compress::{context_capsules, token_estimate};

const RESPONSE_RESERVE_PERCENT: u64 = 15;
const RESPONSE_RESERVE_MIN: u64 = 4_096;
const RESPONSE_RESERVE_MAX: u64 = 16_384;
const CHARS_PER_TOKEN: usize = 4;
const REQUIRED_CONTEXT_ERROR: &str = "Le rapport du sous-agent dépasse la capacité du modèle.";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextBudgetReport {
    pub estimated_tokens: usize,
    pub max_input_tokens: Option<usize>,
    pub pruned_messages: usize,
}

pub fn prepare_for_request(
    messages: &mut Vec<ChatMessage>,
    context_window: u64,
    tools: &[serde_json::Value],
    provider_id: &str,
) -> Result<ContextBudgetReport, String> {
    let tool_tokens = token_estimate::estimate_tool_tokens(tools);
    let estimated =
        token_estimate::estimate_request_tokens_for_provider(provider_id, messages, tools);
    let Some(max_input) = max_input_tokens(context_window) else {
        return Ok(ContextBudgetReport {
            estimated_tokens: estimated,
            max_input_tokens: None,
            pruned_messages: 0,
        });
    };
    prepare_with_limit(
        messages,
        max_input,
        tool_tokens,
        context_window,
        provider_id,
    )
}

pub fn reduce_after_payload_too_large(
    messages: &mut Vec<ChatMessage>,
    context_window: u64,
    tools: &[serde_json::Value],
    provider_id: &str,
) -> Result<bool, String> {
    let tool_tokens = token_estimate::estimate_tool_tokens(tools);
    let before =
        token_estimate::estimate_request_tokens_for_provider(provider_id, messages, tools);
    let configured_limit = max_input_tokens(context_window).unwrap_or(before);
    let reduced_limit = before.saturating_mul(3).saturating_div(4);
    let target = configured_limit.min(reduced_limit);
    if target <= tool_tokens {
        return Ok(false);
    }
    let report = prepare_with_limit(messages, target, tool_tokens, target as u64, provider_id)?;
    Ok(report.estimated_tokens < before)
}

fn prepare_with_limit(
    messages: &mut Vec<ChatMessage>,
    max_input: usize,
    tool_tokens: usize,
    capsule_context: u64,
    provider_id: &str,
) -> Result<ContextBudgetReport, String> {
    let estimated = estimate_messages(provider_id, messages).saturating_add(tool_tokens);
    if estimated <= max_input {
        return Ok(ContextBudgetReport {
            estimated_tokens: estimated,
            max_input_tokens: Some(max_input),
            pruned_messages: 0,
        });
    }

    let message_limit = max_input.saturating_sub(tool_tokens);
    let original_len = messages.len();
    let mut next: Vec<ChatMessage> = messages
        .iter()
        .filter(|m| m.role == "system")
        .cloned()
        .collect();
    let required_reports = messages
        .iter()
        .filter(|message| is_required_report(message))
        .cloned()
        .collect::<Vec<_>>();
    let required_tokens = estimate_messages(provider_id, &next)
        .saturating_add(estimate_messages(provider_id, &required_reports));
    if required_tokens > message_limit {
        return Err(REQUIRED_CONTEXT_ERROR.to_string());
    }

    let capsule = context_capsules::recent_file_context_message(messages, capsule_context)
        .filter(|message| {
            required_tokens.saturating_add(estimate_messages(
                provider_id,
                std::slice::from_ref(message),
            )) <= message_limit
        });
    context_capsules::insert_after_system(&mut next, capsule);

    let mut remaining_budget = message_limit
        .saturating_sub(estimate_messages(provider_id, &next))
        .saturating_sub(estimate_messages(provider_id, &required_reports));
    let mut tail = Vec::new();
    for msg in messages
        .iter()
        .rev()
        .filter(|m| m.role != "system" && !is_required_report(m))
    {
        if remaining_budget == 0 {
            break;
        }
        let msg_tokens = estimate_messages(provider_id, std::slice::from_ref(msg));
        if msg_tokens <= remaining_budget {
            tail.push(msg.clone());
            remaining_budget -= msg_tokens;
        } else if tail.is_empty() {
            tail.push(trim_message(msg, remaining_budget));
            remaining_budget = 0;
        }
    }
    tail.reverse();
    next.extend(tail);
    next.extend(required_reports);
    *messages = next;

    Ok(ContextBudgetReport {
        estimated_tokens: estimate_messages(provider_id, messages).saturating_add(tool_tokens),
        max_input_tokens: Some(max_input),
        pruned_messages: original_len.saturating_sub(messages.len()),
    })
}

fn estimate_messages(provider_id: &str, messages: &[ChatMessage]) -> usize {
    token_estimate::estimate_tokens_for_provider(provider_id, messages)
}

fn is_required_report(message: &ChatMessage) -> bool {
    message
        .content
        .starts_with(super::subagent_report_context::SUBAGENT_REPORT_CONTEXT_PREFIX)
}

pub fn max_input_tokens(context_window: u64) -> Option<usize> {
    if context_window == 0 {
        return None;
    }
    let reserve = response_reserve(context_window).min(context_window / 2);
    Some(context_window.saturating_sub(reserve) as usize)
}

fn response_reserve(context_window: u64) -> u64 {
    let target = context_window.saturating_mul(RESPONSE_RESERVE_PERCENT) / 100;
    target.clamp(RESPONSE_RESERVE_MIN, RESPONSE_RESERVE_MAX)
}

fn trim_message(msg: &ChatMessage, max_tokens: usize) -> ChatMessage {
    let max_chars = max_tokens.saturating_mul(CHARS_PER_TOKEN);
    let mut trimmed = msg.clone();
    trimmed.content = truncate_chars(&msg.content, max_chars);
    trimmed.images = None;
    trimmed.tool_calls = None;
    trimmed
}

fn truncate_chars(input: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return "[message omitted: context budget exhausted]".to_string();
    }
    if input.chars().count() <= max_chars {
        return input.to_string();
    }
    let kept: String = input.chars().take(max_chars).collect();
    format!("{kept}\n[message truncated for context budget]")
}

#[cfg(test)]
#[path = "context_budget_tests.rs"]
mod tests;
