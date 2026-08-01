use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::compress::{context_capsules, token_estimate};

const REQUIRED_CONTEXT_ERROR: &str = "Le rapport du sous-agent dépasse la capacité du modèle.";
const TRUNCATION_NOTICE: &str = "\n[message truncated for context budget]";

pub(super) struct PruneParams<'a> {
    pub max_input: usize,
    pub tool_tokens: usize,
    pub capsule_context: u64,
    pub provider_id: &'a str,
    pub original_len: usize,
    pub repair: super::context_budget_history::HistoryRepairReport,
}

pub(super) fn prepare_with_limit(
    messages: &mut Vec<ChatMessage>,
    params: PruneParams<'_>,
) -> Result<super::context_budget::ContextBudgetReport, String> {
    let estimated = estimate_total(messages, &params);
    if estimated <= params.max_input {
        return Ok(final_report(estimated, messages.len(), &params));
    }

    let message_limit = params.max_input.saturating_sub(params.tool_tokens);
    let mut next: Vec<ChatMessage> = messages
        .iter()
        .filter(|message| message.role == "system")
        .cloned()
        .collect();
    let required_reports = messages
        .iter()
        .filter(|message| is_required_report(message))
        .cloned()
        .collect::<Vec<_>>();
    let required_tokens = super::context_budget::estimate_messages(params.provider_id, &next)
        .saturating_add(super::context_budget::estimate_messages(
            params.provider_id,
            &required_reports,
        ));
    if required_tokens > message_limit {
        return Err(REQUIRED_CONTEXT_ERROR.to_string());
    }

    let capsule = context_capsules::recent_file_context_message(messages, params.capsule_context)
        .filter(|message| {
            required_tokens.saturating_add(super::context_budget::estimate_messages(
                params.provider_id,
                std::slice::from_ref(message),
            )) <= message_limit
        });
    context_capsules::insert_after_system(&mut next, capsule);
    let remaining = message_limit
        .saturating_sub(super::context_budget::estimate_messages(params.provider_id, &next))
        .saturating_sub(super::context_budget::estimate_messages(
            params.provider_id,
            &required_reports,
        ));

    let candidates = messages
        .iter()
        .filter(|message| message.role != "system" && !is_required_report(message))
        .collect();
    append_recent_tail(&mut next, candidates, remaining, params.provider_id);
    next.extend(required_reports);
    *messages = next;

    Ok(final_report(estimate_total(messages, &params), messages.len(), &params))
}

fn append_recent_tail<'a>(
    next: &mut Vec<ChatMessage>,
    candidates: Vec<&'a ChatMessage>,
    mut remaining: usize,
    provider_id: &str,
) {
    let mut selected: Vec<Vec<&'a ChatMessage>> = Vec::new();
    let mut trimmed = None;
    for unit in super::context_budget_history::atomic_units(candidates)
        .into_iter()
        .rev()
    {
        if !unit.valid {
            continue;
        }
        if remaining == 0 {
            break;
        }
        let tokens = estimate_refs(provider_id, &unit.messages);
        if tokens <= remaining {
            selected.push(unit.messages);
            remaining -= tokens;
        } else if selected.is_empty() && !unit.is_tool_chain {
            trimmed = Some(trim_message(unit.messages[0], remaining));
            break;
        } else {
            // Conserve un suffixe chronologique continu : pas de saut derrière un tour trop grand.
            break;
        }
    }
    if let Some(message) = trimmed {
        next.push(message);
        return;
    }
    selected.reverse();
    next.extend(selected.into_iter().flatten().cloned());
}

fn estimate_refs(provider_id: &str, messages: &[&ChatMessage]) -> usize {
    messages.iter().fold(0usize, |total, message| {
        total.saturating_add(token_estimate::estimate_message_tokens_for_provider(
            provider_id,
            message,
        ))
    })
}

fn estimate_total(messages: &[ChatMessage], params: &PruneParams<'_>) -> usize {
    super::context_budget::estimate_messages(params.provider_id, messages)
        .saturating_add(params.tool_tokens)
}

fn final_report(
    estimated: usize,
    final_len: usize,
    params: &PruneParams<'_>,
) -> super::context_budget::ContextBudgetReport {
    super::context_budget::report(
        estimated,
        Some(params.max_input),
        params.original_len,
        final_len,
        params.repair,
    )
}

fn is_required_report(message: &ChatMessage) -> bool {
    message
        .content
        .starts_with(super::subagent_report_context::SUBAGENT_REPORT_CONTEXT_PREFIX)
}

fn trim_message(message: &ChatMessage, max_tokens: usize) -> ChatMessage {
    let mut trimmed = message.clone();
    trimmed.content = truncate_content(&message.content, max_tokens);
    trimmed.images = None;
    trimmed.tool_calls = None;
    trimmed.reasoning_content = None;
    trimmed
}

fn truncate_content(input: &str, max_tokens: usize) -> String {
    let max_units = crate::services::token_counting::max_text_units(max_tokens);
    if crate::services::token_counting::text_units(input) <= max_units {
        return input.to_string();
    }
    let notice_units = crate::services::token_counting::text_units(TRUNCATION_NOTICE);
    if notice_units > max_units {
        return prefix_within_units(TRUNCATION_NOTICE.trim_start(), max_units);
    }
    let kept = prefix_within_units(input, max_units - notice_units);
    format!("{kept}{TRUNCATION_NOTICE}")
}

fn prefix_within_units(input: &str, max_units: usize) -> String {
    let mut used = 0usize;
    input
        .chars()
        .take_while(|character| {
            let mut encoded = [0u8; 4];
            let units = crate::services::token_counting::text_units(
                character.encode_utf8(&mut encoded),
            );
            let fits = used.saturating_add(units) <= max_units;
            if fits {
                used += units;
            }
            fits
        })
        .collect()
}

#[cfg(test)]
#[path = "context_budget_prune_tests.rs"]
mod tests;
