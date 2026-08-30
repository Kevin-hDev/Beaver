use super::profile_resolve::ResolvedCompressionProfile;
use super::profile_types::CompressionWindowBand;

pub fn projected_budget(
    profile: &ResolvedCompressionProfile,
    context_window: u64,
    before_tokens: u32,
) -> u32 {
    let band = match profile.band(context_window) {
        Some(CompressionWindowBand::Under64K) => &profile.profile.under_64k,
        Some(CompressionWindowBand::Large) => &profile.profile.large,
        Some(CompressionWindowBand::Compact) | None => &profile.profile.compact,
    };
    let window = super::profile_budget::effective_budget_window(context_window, before_tokens);
    let target = ((u128::from(window) * u128::from(band.target_percent)) / 100)
        .min(u128::from(u32::MAX)) as u32;
    target.saturating_sub(super::profile_budget::resolve_budget(
        &band.response_reserve,
        window,
    ))
}
