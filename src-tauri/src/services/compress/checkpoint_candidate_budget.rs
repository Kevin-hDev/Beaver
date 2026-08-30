use super::checkpoint_document::CheckpointSection;
use super::checkpoint_selection::CheckpointSelectionLimits;
use super::profile_types::CompressionBandSettings;
use super::snapshot::CompressionSnapshot;
use super::summary_contract::ValidatedSummary;

pub(super) fn selection_limits(
    snapshot: &CompressionSnapshot,
    band: &CompressionBandSettings,
    summary: Option<&ValidatedSummary>,
    sections: &[CheckpointSection],
    evidence_tokens: u32,
) -> CheckpointSelectionLimits {
    let window = budget_window(snapshot);
    let evidence = super::profile_budget::resolve_budget(&band.evidence_envelope, window);
    let section_tokens = sections.iter().fold(0u32, |total, section| {
        total.saturating_add(
            crate::services::token_counting::estimate_text_tokens(&section.content)
                .min(u32::MAX as usize) as u32,
        )
    });
    let checkpoint_overhead =
        section_tokens.saturating_add(summary.map_or(0, |value| value.estimated_tokens));
    let remaining_evidence = evidence.saturating_sub(evidence_tokens);
    CheckpointSelectionLimits {
        user_tokens: category_tokens(&band.user_messages, window),
        assistant_tokens: category_tokens(&band.assistant_messages, window),
        tool_tokens: if !band.tools.enabled {
            0
        } else if band.tools.total_tokens > 0 {
            band.tools.total_tokens.min(remaining_evidence)
        } else {
            remaining_evidence
        },
        tool_tokens_per_result: band.tools.tokens_per_item,
        max_tool_events: if band.tools.enabled {
            band.tools.max_items
        } else {
            0
        },
        total_tokens: target_tokens(window, band)
            .saturating_sub(reserve_tokens(window, band))
            .saturating_sub(checkpoint_overhead),
    }
}

pub(super) fn target_tokens(window: u64, band: &CompressionBandSettings) -> u32 {
    ((window as u128 * u128::from(band.target_percent)) / 100).min(u128::from(u32::MAX)) as u32
}

pub(super) fn reserve_tokens(window: u64, band: &CompressionBandSettings) -> u32 {
    super::profile_budget::resolve_budget(&band.response_reserve, window)
}

fn budget_window(snapshot: &CompressionSnapshot) -> u64 {
    super::profile_budget::effective_budget_window(snapshot.context_window, snapshot.before_tokens)
}

fn category_tokens(budget: &super::profile_types::CategoryBudget, window: u64) -> u32 {
    if budget.enabled {
        super::profile_budget::resolve_budget(&budget.tokens, window)
    } else {
        0
    }
}
