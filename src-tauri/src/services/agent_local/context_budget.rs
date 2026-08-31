use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::compress::token_estimate;

const RESPONSE_RESERVE_PERCENT: u64 = 15;
const RESPONSE_RESERVE_MIN: u64 = 4_096;
const RESPONSE_RESERVE_MAX: u64 = 16_384;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextBudgetReport {
    pub estimated_tokens: usize,
    pub max_input_tokens: Option<usize>,
    pub pruned_messages: usize,
    pub repaired_tool_chains: usize,
    pub dropped_tool_results: usize,
}

pub fn prepare_for_request(
    messages: &mut Vec<ChatMessage>,
    context_window: u64,
    tools: &[serde_json::Value],
    provider_id: &str,
) -> Result<ContextBudgetReport, String> {
    let original_len = messages.len();
    let repair = super::context_budget_history::repair_invalid_history(messages);
    let tool_tokens = token_estimate::estimate_tool_tokens(tools);
    let estimated = estimate_messages(provider_id, messages).saturating_add(tool_tokens);
    let Some(max_input) = max_input_tokens(context_window) else {
        return Ok(report(
            estimated,
            None,
            original_len,
            messages.len(),
            repair,
        ));
    };
    super::context_budget_prune::prepare_with_limit(
        messages,
        super::context_budget_prune::PruneParams {
            max_input,
            tool_tokens,
            context_window,
            provider_id,
            original_len,
            repair,
        },
    )
}

pub fn reduce_after_payload_too_large(
    messages: &mut Vec<ChatMessage>,
    context_window: u64,
    tools: &[serde_json::Value],
    provider_id: &str,
) -> Result<bool, String> {
    let before = token_estimate::estimate_request_tokens_for_provider(provider_id, messages, tools);
    let original_len = messages.len();
    let repair = super::context_budget_history::repair_invalid_history(messages);
    let repaired = original_len != messages.len() || repair.repaired_tool_chains > 0;
    let tool_tokens = token_estimate::estimate_tool_tokens(tools);
    let configured_limit = max_input_tokens(context_window).unwrap_or(before);
    let target = configured_limit.min(before.saturating_mul(3).saturating_div(4));
    if target <= tool_tokens {
        return Ok(repaired);
    }
    let report = super::context_budget_prune::prepare_with_limit(
        messages,
        super::context_budget_prune::PruneParams {
            max_input: target,
            tool_tokens,
            context_window,
            provider_id,
            original_len,
            repair,
        },
    )?;
    Ok(repaired || report.estimated_tokens < before)
}

pub(super) fn report(
    estimated_tokens: usize,
    max_input_tokens: Option<usize>,
    original_len: usize,
    final_len: usize,
    repair: super::context_budget_history::HistoryRepairReport,
) -> ContextBudgetReport {
    ContextBudgetReport {
        estimated_tokens,
        max_input_tokens,
        pruned_messages: original_len.saturating_sub(final_len),
        repaired_tool_chains: repair.repaired_tool_chains,
        dropped_tool_results: repair.dropped_tool_results,
    }
}

pub(super) fn estimate_messages(provider_id: &str, messages: &[ChatMessage]) -> usize {
    token_estimate::estimate_tokens_for_provider(provider_id, messages)
}

pub async fn record_repairs(report: &ContextBudgetReport, session_id: &str, request_id: &str) {
    if report.repaired_tool_chains == 0 && report.dropped_tool_results == 0 {
        return;
    }
    let message = format!(
        "Historique outils réparé: {} chaîne(s), {} résultat(s) orphelin(s) retiré(s).",
        report.repaired_tool_chains, report.dropped_tool_results
    );
    super::stream_diagnostics::mark_phase(
        session_id,
        request_id,
        "context_history_repaired",
        &message,
    )
    .await;
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

#[cfg(test)]
#[path = "context_budget_tests.rs"]
mod tests;
