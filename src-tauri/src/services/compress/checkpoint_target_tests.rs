use super::checkpoint_target::{
    checkpoint_target, effective_summary_limit, preview_bucket, SummaryCapacityError,
};
use super::profile_types::CompressionWindowBand;

#[test]
fn target_uses_real_textual_usage_and_floor_rounding() {
    for (before, head, band, expected) in [
        (96_000, 12_000, CompressionWindowBand::Compact, 28_800),
        (258_000, 12_000, CompressionWindowBand::Large, 40_000),
        (16_000, 4_000, CompressionWindowBand::Under64K, 6_400),
        (8_192, 3_000, CompressionWindowBand::Under64K, 4_038),
        (10_000, 12_000, CompressionWindowBand::Compact, 12_000),
    ] {
        assert_eq!(checkpoint_target(before, head, band), expected);
    }
}

#[test]
fn preview_uses_eight_thousand_token_buckets() {
    assert_eq!(preview_bucket(28_800), (24_000, 32_000));
    assert_eq!(preview_bucket(40_000), (32_000, 40_000));
}

#[test]
fn summary_limit_accepts_512_but_refuses_511_before_network() {
    assert_eq!(effective_summary_limit(6_000, 4_000), Ok(4_000));
    assert_eq!(effective_summary_limit(2_000, 800), Ok(800));
    assert_eq!(effective_summary_limit(2_000, 512), Ok(512));
    assert_eq!(
        effective_summary_limit(2_000, 511),
        Err(SummaryCapacityError::Insufficient)
    );
}
