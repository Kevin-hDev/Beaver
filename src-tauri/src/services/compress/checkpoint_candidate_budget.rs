use super::checkpoint_document::CheckpointSection;
use super::checkpoint_selection::CheckpointSelectionLimits;
use super::profile_types::{CompressionBandSettings, CompressionWindowBand};
use super::snapshot::CompressionSnapshot;
use super::summary_contract::ValidatedSummary;

const CHECKPOINT_OVERHEAD_TOKENS: u32 = 256;

pub(super) fn selection_limits(
    snapshot: &CompressionSnapshot,
    kind: CompressionWindowBand,
    band: &CompressionBandSettings,
    summary: &ValidatedSummary,
    sections: &[CheckpointSection],
    evidence_tokens: u32,
) -> CheckpointSelectionLimits {
    let section_tokens = sections.iter().fold(0u32, |total, section| {
        total.saturating_add(
            crate::services::token_counting::estimate_text_tokens(&section.content)
                .min(u32::MAX as usize) as u32,
        )
    });
    let total_tokens = available_after_summary(snapshot, kind, summary.estimated_tokens)
        .saturating_sub(section_tokens);
    let tool_tokens = super::checkpoint_evidence::envelope_tokens(kind)
        .saturating_sub(evidence_tokens)
        .min(total_tokens);
    let tool_tokens_per_result = if band.tool_result_count == 0 {
        0
    } else {
        (tool_tokens / u32::from(band.tool_result_count)).clamp(128, 2_000)
    };
    CheckpointSelectionLimits {
        recent_message_count: band.recent_message_count,
        tool_tokens,
        tool_tokens_per_result,
        max_tool_events: band.tool_result_count,
        total_tokens,
    }
}

pub(super) fn available_after_summary(
    snapshot: &CompressionSnapshot,
    kind: CompressionWindowBand,
    summary_tokens: u32,
) -> u32 {
    target_tokens(snapshot, kind)
        .saturating_sub(snapshot.system_head_tokens)
        .saturating_sub(summary_tokens)
        .saturating_sub(CHECKPOINT_OVERHEAD_TOKENS)
}

pub(super) fn target_tokens(snapshot: &CompressionSnapshot, kind: CompressionWindowBand) -> u32 {
    super::checkpoint_target::checkpoint_target(
        snapshot.before_tokens,
        snapshot.system_head_tokens,
        kind,
    )
}
