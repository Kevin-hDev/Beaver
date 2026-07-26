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
) -> Result<ContextBudgetReport, String> {
    let estimated = token_estimate::estimate_tokens(messages);
    let Some(max_input) = max_input_tokens(context_window) else {
        return Ok(ContextBudgetReport {
            estimated_tokens: estimated,
            max_input_tokens: None,
            pruned_messages: 0,
        });
    };
    if estimated <= max_input {
        return Ok(ContextBudgetReport {
            estimated_tokens: estimated,
            max_input_tokens: Some(max_input),
            pruned_messages: 0,
        });
    }

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
    let required_tokens = token_estimate::estimate_tokens(&next)
        .saturating_add(token_estimate::estimate_tokens(&required_reports));
    if required_tokens > max_input {
        return Err(REQUIRED_CONTEXT_ERROR.to_string());
    }

    let capsule = context_capsules::recent_file_context_message(messages, context_window)
        .filter(|message| {
            required_tokens.saturating_add(token_estimate::estimate_tokens(
                std::slice::from_ref(message),
            )) <= max_input
        });
    context_capsules::insert_after_system(&mut next, capsule);

    let mut remaining_budget = max_input
        .saturating_sub(token_estimate::estimate_tokens(&next))
        .saturating_sub(token_estimate::estimate_tokens(&required_reports));
    let mut tail = Vec::new();
    for msg in messages
        .iter()
        .rev()
        .filter(|m| m.role != "system" && !is_required_report(m))
    {
        if remaining_budget == 0 {
            break;
        }
        let msg_tokens = token_estimate::estimate_tokens(std::slice::from_ref(msg));
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
        estimated_tokens: token_estimate::estimate_tokens(messages),
        max_input_tokens: Some(max_input),
        pruned_messages: original_len.saturating_sub(messages.len()),
    })
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
