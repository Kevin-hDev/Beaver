use super::profile_types::CompressionWindowBand;

#[derive(Debug, Clone, Copy)]
pub struct EvidenceItemLimit {
    pub max_items: u16,
    pub tokens_per_item: u32,
    pub total_tokens: u32,
}

pub const fn envelope_tokens(band: CompressionWindowBand) -> u32 {
    match band {
        CompressionWindowBand::Under64K => 2_000,
        CompressionWindowBand::Compact => 4_000,
        CompressionWindowBand::Large => 6_000,
    }
}

pub fn item_limit(max_items: u16, remaining: u32) -> EvidenceItemLimit {
    let divisor = u32::from(max_items.max(1));
    EvidenceItemLimit {
        max_items,
        tokens_per_item: (remaining / divisor).clamp(256, 8_000),
        total_tokens: remaining,
    }
}
