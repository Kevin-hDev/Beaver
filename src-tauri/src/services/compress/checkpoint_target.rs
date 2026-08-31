use super::profile_types::CompressionWindowBand;

pub const MIN_EFFECTIVE_SUMMARY_TOKENS: u32 = 512;
const PREVIEW_BUCKET_TOKENS: u32 = 8_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SummaryCapacityError {
    Insufficient,
}

pub fn checkpoint_target(before_tokens: u32, head_tokens: u32, band: CompressionWindowBand) -> u32 {
    let compressible = before_tokens.saturating_sub(head_tokens);
    let percentage = compressible.saturating_mul(20) / 100;
    head_tokens.saturating_add(percentage.min(variable_cap(band)))
}

pub fn effective_summary_limit(
    profile_limit: u32,
    available_tokens: u32,
) -> Result<u32, SummaryCapacityError> {
    let limit = profile_limit.min(available_tokens);
    if limit < MIN_EFFECTIVE_SUMMARY_TOKENS {
        Err(SummaryCapacityError::Insufficient)
    } else {
        Ok(limit)
    }
}

pub fn preview_bucket(target_tokens: u32) -> (u32, u32) {
    let upper = target_tokens.saturating_add(PREVIEW_BUCKET_TOKENS - 1) / PREVIEW_BUCKET_TOKENS
        * PREVIEW_BUCKET_TOKENS;
    let upper = upper.max(PREVIEW_BUCKET_TOKENS);
    (upper.saturating_sub(PREVIEW_BUCKET_TOKENS), upper)
}

pub const fn variable_cap(band: CompressionWindowBand) -> u32 {
    match band {
        CompressionWindowBand::Under64K => 10_000,
        CompressionWindowBand::Compact => 20_000,
        CompressionWindowBand::Large => 28_000,
    }
}
