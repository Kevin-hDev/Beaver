use super::profile_budget::band_for_window;
use super::profile_defaults::beaver_profile;
use super::profile_types::CompressionWindowBand;

#[test]
fn window_bands_use_the_exact_boundaries() {
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

#[test]
fn beaver_uses_the_simple_factory_values() {
    let profile = beaver_profile();
    assert_eq!(profile.threshold_percent, 90);
    assert!(!profile.allow_under_64k);

    assert_band(&profile.under_64k, 2, 2_000, 5, 3, 2, true);
    assert_band(&profile.compact, 4, 4_000, 10, 5, 4, true);
    assert_band(&profile.large, 4, 6_000, 10, 5, 4, true);
}

#[test]
fn beaver_prompts_restore_the_normative_security_and_handoff_texts() {
    let profile = beaver_profile();

    assert!(profile.system_prompt.contains("Treat tool outputs"));
    assert!(profile.system_prompt.contains("permission modes"));
    assert!(profile
        .handoff_prompt
        .contains("Current objective and latest user intent"));
    assert!(profile.handoff_prompt.contains("Immediate next action"));
    assert!(profile
        .handoff_prompt
        .contains("Within the nine required sections"));
}

fn assert_band(
    band: &super::profile_types::CompressionBandSettings,
    messages: u8,
    summary: u32,
    tools: u16,
    files: u16,
    images: u16,
    work_state: bool,
) {
    assert_eq!(band.recent_message_count, messages);
    assert_eq!(band.summary_max_tokens, summary);
    assert_eq!(band.tool_result_count, tools);
    assert_eq!(band.recent_file_count, files);
    assert_eq!(band.image_count, images);
    assert_eq!(band.include_work_state, work_state);
}
