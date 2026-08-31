use crate::models::compression_profile_contract::BudgetProjectionView;
use crate::services::compress::profile_limits::{
    SETTINGS_CONTEXT_TOKENS, SETTINGS_SYSTEM_TOOLS_TOKENS,
};
use crate::services::compress::profile_store::CompressionProfileStoreError;
use crate::services::compress::profile_types::{CompressionProfile, CompressionWindowBand};

pub(super) fn project(
    profile: &CompressionProfile,
    band: CompressionWindowBand,
) -> Result<BudgetProjectionView, CompressionProfileStoreError> {
    let settings = match band {
        CompressionWindowBand::Under64K => &profile.under_64k,
        CompressionWindowBand::Compact => &profile.compact,
        CompressionWindowBand::Large => &profile.large,
    };
    let target_tokens = crate::services::compress::checkpoint_target::checkpoint_target(
        SETTINGS_CONTEXT_TOKENS,
        SETTINGS_SYSTEM_TOOLS_TOKENS,
        band,
    );
    let variable_tokens = target_tokens.saturating_sub(SETTINGS_SYSTEM_TOOLS_TOKENS);
    let (range_lower_tokens, range_upper_tokens) =
        crate::services::compress::checkpoint_target::preview_bucket(target_tokens);
    let projected_percent = target_tokens
        .saturating_mul(100)
        .checked_div(SETTINGS_CONTEXT_TOKENS)
        .unwrap_or_default()
        .min(100) as u8;
    Ok(BudgetProjectionView {
        band,
        before_tokens: SETTINGS_CONTEXT_TOKENS,
        system_tools_tokens: SETTINGS_SYSTEM_TOOLS_TOKENS,
        variable_tokens,
        target_tokens,
        range_lower_tokens,
        range_upper_tokens,
        image_count: settings.image_count,
        projected_percent,
    })
}
