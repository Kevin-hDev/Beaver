use super::profile_budget::band_for_window;
use super::profile_types::CompressionWindowBand;

#[test]
fn window_bands_keep_the_exact_product_boundaries() {
    assert_eq!(band_for_window(0), None);
    assert_eq!(
        band_for_window(63_999),
        Some(CompressionWindowBand::Under64K)
    );
    assert_eq!(
        band_for_window(64_000),
        Some(CompressionWindowBand::Compact)
    );
    assert_eq!(
        band_for_window(127_999),
        Some(CompressionWindowBand::Compact)
    );
    assert_eq!(band_for_window(128_000), Some(CompressionWindowBand::Large));
}
