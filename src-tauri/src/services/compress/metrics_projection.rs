use super::profile_resolve::ResolvedCompressionProfile;
use super::profile_types::CompressionWindowBand;

pub fn projected_target(
    profile: &ResolvedCompressionProfile,
    context_window: u64,
    before_tokens: u32,
    system_head_tokens: u32,
) -> u32 {
    let band = profile
        .band(context_window)
        .unwrap_or(CompressionWindowBand::Compact);
    super::checkpoint_target::checkpoint_target(before_tokens, system_head_tokens, band)
}
