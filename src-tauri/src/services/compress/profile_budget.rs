use super::profile_types::CompressionWindowBand;

pub fn band_for_window(context_window: u64) -> Option<CompressionWindowBand> {
    if context_window == 0 {
        None
    } else if context_window < super::profile_limits::UNDER_64K_UPPER_EXCLUSIVE {
        Some(CompressionWindowBand::Under64K)
    } else if context_window < super::profile_limits::COMPACT_UPPER_EXCLUSIVE {
        Some(CompressionWindowBand::Compact)
    } else {
        Some(CompressionWindowBand::Large)
    }
}
