use super::profile_defaults::beaver_profile;
use super::profile_limits::{
    MAX_BUDGET_TOKENS, MAX_CATEGORY_ITEMS, MAX_CUSTOM_PROMPT_CHARS, MAX_PROFILES,
    MAX_PROFILE_NAME_CHARS, MAX_RETRIES,
};
use super::profile_validation::{normalize_profile_document, validate_profile_input};

#[test]
fn absolute_limits_are_centralized_and_distinct_from_defaults() {
    assert_eq!(MAX_PROFILES, 20);
    assert_eq!(MAX_PROFILE_NAME_CHARS, 48);
    assert_eq!(MAX_CUSTOM_PROMPT_CHARS, 32_000);
    assert_eq!(MAX_CATEGORY_ITEMS, 100);
    assert_eq!(MAX_BUDGET_TOKENS, 1_000_000);
    assert_eq!(MAX_RETRIES, 2);

    let profile = beaver_profile();
    assert_eq!(profile.compact.images.max_items, 16);
    assert_eq!(profile.compact.critical_references.max_items, 32);
}

#[test]
fn ipc_validation_rejects_invalid_identity_names_and_duplicates() {
    let existing = vec![beaver_profile()];
    let mut input = beaver_profile();
    input.id = "not-a-uuid".into();
    input.name = "Custom".into();
    assert!(validate_profile_input(&input, &existing).is_err());

    input.id = "4f93ca54-5c44-44ec-bd90-122fcea4e181".into();
    input.name = "\u{0007}".into();
    assert!(validate_profile_input(&input, &existing).is_err());

    input.name = "BEAVER".into();
    assert!(validate_profile_input(&input, &existing).is_err());
}

#[test]
fn ipc_validation_rejects_oversized_collections_and_values() {
    let mut input = beaver_profile();
    input.id = "4f93ca54-5c44-44ec-bd90-122fcea4e181".into();
    input.name = "x".repeat(MAX_PROFILE_NAME_CHARS + 1);
    assert!(validate_profile_input(&input, &[]).is_err());

    input.name = "Custom".into();
    input.summary.system_prompt = "x".repeat(MAX_CUSTOM_PROMPT_CHARS + 1);
    assert!(validate_profile_input(&input, &[]).is_err());

    input.summary.system_prompt.clear();
    input.compact.tools.max_items = MAX_CATEGORY_ITEMS + 1;
    assert!(validate_profile_input(&input, &[]).is_err());
}

#[test]
fn disk_normalization_clamps_values_without_reusing_migration_semantics() {
    let mut profiles = vec![beaver_profile()];
    profiles[0].threshold_percent = 0;
    profiles[0].summary.ordinary_retries = u8::MAX;
    profiles[0].compact.tools.tokens_per_item = MAX_BUDGET_TOKENS + 1;
    let mut global_id = "missing".to_string();

    normalize_profile_document(&mut profiles, &mut global_id);

    assert_eq!(profiles[0].threshold_percent, 1);
    assert_eq!(profiles[0].summary.ordinary_retries, MAX_RETRIES);
    assert_eq!(profiles[0].compact.tools.tokens_per_item, MAX_BUDGET_TOKENS);
    assert_eq!(global_id, "beaver");
}

#[test]
fn profile_collection_is_bounded() {
    let mut profiles = (0..=MAX_PROFILES)
        .map(|index| {
            let mut profile = beaver_profile();
            profile.id = format!("00000000-0000-4000-8000-{index:012}");
            profile.name = format!("Profile {index}");
            profile
        })
        .collect::<Vec<_>>();
    let mut global_id = profiles.last().expect("last profile").id.clone();

    normalize_profile_document(&mut profiles, &mut global_id);

    assert_eq!(profiles.len(), MAX_PROFILES);
    assert_eq!(global_id, "beaver");
}
