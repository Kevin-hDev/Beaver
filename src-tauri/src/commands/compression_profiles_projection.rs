use crate::models::compression_profile_contract::BudgetProjectionView;
use crate::services::compress::profile_budget::{resolve_budget, summary_output_limit};
use crate::services::compress::profile_limits::{
    SETTINGS_IMAGE_TOKEN_ESTIMATE, SETTINGS_SYSTEM_TOOLS_ESTIMATE,
};
use crate::services::compress::profile_store::CompressionProfileStoreError;
use crate::services::compress::profile_types::{
    CategoryBudget, CompressionBandSettings, CompressionProfile, CompressionWindowBand,
};

pub(super) fn project(
    profile: &CompressionProfile,
    band: CompressionWindowBand,
    context_window: u64,
) -> Result<BudgetProjectionView, CompressionProfileStoreError> {
    if crate::services::compress::profile_budget::band_for_window(context_window) != Some(band) {
        return Err(CompressionProfileStoreError::Invalid);
    }
    let settings = match band {
        CompressionWindowBand::Under64K => &profile.under_64k,
        CompressionWindowBand::Compact => &profile.compact,
        CompressionWindowBand::Large => &profile.large,
    };
    let threshold_input = context_window.saturating_mul(u64::from(profile.threshold_percent)) / 100;
    let summary_tokens = if profile.summary.enabled {
        summary_output_limit(&settings.summary_output, context_window, threshold_input)
    } else {
        0
    };
    let categories_tokens = category_total(settings, context_window);
    let reserve_tokens = resolve_budget(&settings.response_reserve, context_window);
    let total_tokens = u64::from(SETTINGS_SYSTEM_TOOLS_ESTIMATE)
        .saturating_add(u64::from(summary_tokens))
        .saturating_add(categories_tokens)
        .saturating_add(u64::from(reserve_tokens));
    let projected_percent = total_tokens
        .saturating_mul(100)
        .checked_div(context_window)
        .unwrap_or(u64::MAX)
        .min(100) as u8;
    let exceeds_window = total_tokens > context_window;
    Ok(BudgetProjectionView {
        context_window,
        band,
        system_tools_tokens: SETTINGS_SYSTEM_TOOLS_ESTIMATE,
        summary_tokens,
        categories_tokens: categories_tokens.min(u64::from(u32::MAX)) as u32,
        reserve_tokens,
        total_tokens,
        projected_percent,
        exceeds_window,
        high_risk: exceeds_window || projected_percent >= profile.threshold_percent,
    })
}

fn category_total(settings: &CompressionBandSettings, context_window: u64) -> u64 {
    let messages = enabled_budget(&settings.user_messages, context_window)
        .saturating_add(enabled_budget(&settings.assistant_messages, context_window));
    let has_evidence = settings.tools.enabled
        || settings.files.enabled
        || settings.modified_files.enabled
        || settings.text_attachments.enabled
        || settings.git_tokens.enabled
        || settings.plan_and_tasks_tokens.enabled
        || settings.subagent_detail_tokens.enabled
        || settings.unresolved_state_tokens.enabled
        || settings.critical_references.enabled;
    let evidence = if has_evidence {
        resolve_budget(&settings.evidence_envelope, context_window)
    } else {
        0
    };
    let images = if settings.images.enabled {
        u32::from(settings.images.max_items).saturating_mul(SETTINGS_IMAGE_TOKEN_ESTIMATE)
    } else {
        0
    };
    u64::from(messages)
        .saturating_add(u64::from(evidence))
        .saturating_add(u64::from(images))
}

fn enabled_budget(budget: &CategoryBudget, context_window: u64) -> u32 {
    if budget.enabled {
        resolve_budget(&budget.tokens, context_window)
    } else {
        0
    }
}
