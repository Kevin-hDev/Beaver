use super::profile_defaults::beaver_profile;
use super::profile_limits::{
    MAX_CUSTOM_PROMPT_CHARS, MAX_FILES, MAX_IMAGES, MAX_MESSAGES, MAX_PROFILES,
    MAX_PROFILE_NAME_CHARS, MAX_SUMMARY_TOKENS, MAX_TOOL_RESULTS, MIN_SUMMARY_TOKENS,
};
use super::profile_validation::{normalize_profile_document, validate_profile_input};

#[test]
fn absolute_limits_are_centralized() {
    assert_eq!(MAX_PROFILES, 20);
    assert_eq!(MAX_PROFILE_NAME_CHARS, 48);
    assert_eq!(MAX_CUSTOM_PROMPT_CHARS, 32_000);
    assert_eq!(MAX_MESSAGES, 8);
    assert_eq!(MAX_TOOL_RESULTS, 50);
    assert_eq!(MAX_FILES, 15);
    assert_eq!(MAX_IMAGES, 16);
    assert_eq!(MIN_SUMMARY_TOKENS, 1_000);
    assert_eq!(MAX_SUMMARY_TOKENS, 8_000);
}

#[test]
fn ipc_validation_rejects_out_of_range_simple_quantities() {
    let existing = vec![beaver_profile()];
    let mut input = custom_profile();

    input.compact.recent_message_count = 9;
    assert!(validate_profile_input(&input, &existing).is_err());
    input.compact.recent_message_count = 8;

    input.compact.summary_max_tokens = 999;
    assert!(validate_profile_input(&input, &existing).is_err());
    input.compact.summary_max_tokens = 8_001;
    assert!(validate_profile_input(&input, &existing).is_err());
    input.compact.summary_max_tokens = 1_000;

    input.compact.tool_result_count = 51;
    assert!(validate_profile_input(&input, &existing).is_err());
    input.compact.tool_result_count = 50;

    input.compact.recent_file_count = 16;
    assert!(validate_profile_input(&input, &existing).is_err());
    input.compact.recent_file_count = 15;

    input.compact.image_count = 17;
    assert!(validate_profile_input(&input, &existing).is_err());
    input.compact.image_count = 16;

    assert!(validate_profile_input(&input, &existing).is_ok());
}

#[test]
fn ipc_accepts_each_simple_boundary_and_rejects_threshold_overflows() {
    let existing = vec![beaver_profile()];
    let mut input = custom_profile();
    for threshold in [1, 90] {
        input.threshold_percent = threshold;
        assert!(validate_profile_input(&input, &existing).is_ok());
    }
    for threshold in [0, 91] {
        input.threshold_percent = threshold;
        assert!(validate_profile_input(&input, &existing).is_err());
    }
    input.threshold_percent = 90;
    input.compact.summary_max_tokens = 8_000;
    for messages in [0, 1, 7, 8] {
        input.compact.recent_message_count = messages;
        assert!(validate_profile_input(&input, &existing).is_ok());
    }
}

#[test]
fn disk_normalization_removes_forbidden_prompt_controls() {
    let mut profiles = vec![beaver_profile()];
    profiles[0].system_prompt = "safe\u{000b}text\nkept".into();
    let mut global_id = "beaver".to_string();

    normalize_profile_document(&mut profiles, &mut global_id);

    assert_eq!(profiles[0].system_prompt, "safetext\nkept");
}

#[test]
fn disk_normalization_clamps_simple_values_and_repairs_zero_summary() {
    let mut profiles = vec![beaver_profile()];
    let profile = &mut profiles[0];
    profile.threshold_percent = 0;
    profile.compact.recent_message_count = u8::MAX;
    profile.compact.summary_max_tokens = 0;
    profile.compact.tool_result_count = u16::MAX;
    profile.compact.recent_file_count = u16::MAX;
    profile.compact.image_count = u16::MAX;
    let mut global_id = "missing".to_string();

    normalize_profile_document(&mut profiles, &mut global_id);

    let profile = &profiles[0];
    assert_eq!(profile.threshold_percent, 1);
    assert_eq!(profile.compact.recent_message_count, 8);
    assert_eq!(profile.compact.summary_max_tokens, 1_000);
    assert_eq!(profile.compact.tool_result_count, 50);
    assert_eq!(profile.compact.recent_file_count, 15);
    assert_eq!(profile.compact.image_count, 16);
    assert_eq!(global_id, "beaver");
}

#[test]
fn ipc_validation_rejects_invalid_identity_names_and_prompts() {
    let existing = vec![beaver_profile()];
    let mut input = custom_profile();
    input.id = "not-a-uuid".into();
    assert!(validate_profile_input(&input, &existing).is_err());

    input.id = "4f93ca54-5c44-44ec-bd90-122fcea4e181".into();
    input.name = "BEAVER".into();
    assert!(validate_profile_input(&input, &existing).is_err());

    input.name = "Custom".into();
    input.system_prompt = "x".repeat(MAX_CUSTOM_PROMPT_CHARS + 1);
    assert!(validate_profile_input(&input, &existing).is_err());
}

fn custom_profile() -> super::profile_types::CompressionProfile {
    let mut profile = beaver_profile();
    profile.id = "4f93ca54-5c44-44ec-bd90-122fcea4e181".into();
    profile.name = "Custom".into();
    profile
}
