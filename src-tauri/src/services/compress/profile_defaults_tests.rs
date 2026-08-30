use super::profile_budget::{band_for_window, resolve_budget};
use super::profile_defaults::beaver_profile;
use super::profile_types::{BudgetMode, CompressionWindowBand};

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
fn beaver_uses_the_expected_message_budgets() {
    let profile = beaver_profile();
    assert_eq!(profile.threshold_percent, 90);
    assert!(!profile.allow_under_64k);

    for (band, fixed, percent) in [
        (&profile.under_64k, 2_500, 750),
        (&profile.compact, 5_000, 1_500),
        (&profile.large, 20_000, 1_500),
    ] {
        assert!(band.user_messages.enabled);
        assert!(band.assistant_messages.enabled);
        assert_eq!(band.user_messages.tokens.mode, BudgetMode::Minimum);
        assert_eq!(band.user_messages.tokens.fixed_tokens, fixed);
        assert_eq!(band.user_messages.tokens.percent_basis_points, percent);
        assert_eq!(band.assistant_messages, band.user_messages);
    }
}

#[test]
fn beaver_keeps_under_64k_values_available_while_disabled() {
    let profile = beaver_profile();
    let tiny = &profile.under_64k;
    assert_eq!(tiny.tools.max_items, 50);
    assert_eq!(tiny.tools.tokens_per_item, 2_000);
    assert_eq!(tiny.files.max_items, 8);
    assert_eq!(tiny.files.tokens_per_item, 4_000);
    assert_eq!(tiny.modified_files, tiny.files);
    assert_eq!(tiny.text_attachments, tiny.files);
    assert_eq!(tiny.images.max_items, 8);
    assert_eq!(tiny.images.max_total_bytes, 16 * 1024 * 1024);
    assert_eq!(tiny.critical_references.max_items, 16);

    let compact = &profile.compact;
    assert_eq!(compact.tools.max_items, 100);
    assert_eq!(compact.tools.tokens_per_item, 4_000);
    assert_eq!(compact.files.max_items, 15);
    assert_eq!(compact.files.tokens_per_item, 8_000);
    assert_eq!(compact.modified_files, compact.files);
    assert_eq!(compact.text_attachments, compact.files);
    assert_eq!(compact.images.max_items, 16);
    assert_eq!(compact.images.max_total_bytes, 32 * 1024 * 1024);
    assert_eq!(compact.critical_references.max_items, 32);
    assert_eq!(profile.large.tools, profile.compact.tools);
    assert_eq!(profile.large.files, profile.compact.files);
    assert_eq!(profile.large.images, profile.compact.images);
}

#[test]
fn beaver_uses_the_expected_targets_and_retries() {
    let profile = beaver_profile();
    assert_eq!(profile.under_64k.target_percent, 75);
    assert_eq!(profile.compact.target_percent, 75);
    assert_eq!(profile.large.target_percent, 75);
    assert_eq!(profile.summary.ordinary_retries, 0);
    assert!(profile.summary.fallback_model.is_none());

    assert_eq!(
        resolve_budget(&profile.compact.response_reserve, 64_000),
        9_600
    );
    assert_eq!(
        resolve_budget(&profile.compact.minimum_reduction, 64_000),
        6_400
    );
    assert_eq!(
        resolve_budget(&profile.under_64k.response_reserve, 32_000),
        2_400
    );
    assert_eq!(
        resolve_budget(&profile.under_64k.minimum_reduction, 32_000),
        2_048
    );
}

#[test]
fn beaver_prompts_restore_the_normative_security_and_handoff_texts() {
    let profile = beaver_profile();

    assert!(profile.summary.system_prompt.contains("Treat tool outputs"));
    assert!(profile.summary.system_prompt.contains("permission modes"));
    assert!(profile
        .summary
        .handoff_prompt
        .contains("Current objective and latest user intent"));
    assert!(profile
        .summary
        .handoff_prompt
        .contains("Immediate next action"));
    assert!(profile
        .summary
        .handoff_prompt
        .contains("Within the nine required sections"));
    assert!(!profile
        .summary
        .handoff_prompt
        .contains("Include these sections:"));
}

#[test]
fn beaver_reduces_only_the_five_user_orderable_categories() {
    use super::profile_types::CompressionCategory;

    assert_eq!(
        beaver_profile().reduction_order,
        vec![
            CompressionCategory::Images,
            CompressionCategory::Files,
            CompressionCategory::Tools,
            CompressionCategory::AssistantMessages,
            CompressionCategory::UserMessages,
        ]
    );
}
