use super::profile_types::CompressionWindowBand;

pub fn band_for_window(context_window: u64) -> Option<CompressionWindowBand> {
    match context_window {
        0 => None,
        1..64_000 => Some(CompressionWindowBand::Under64K),
        64_000..128_000 => Some(CompressionWindowBand::Compact),
        _ => Some(CompressionWindowBand::Large),
    }
}
