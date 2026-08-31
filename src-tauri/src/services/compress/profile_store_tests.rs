use serde_json::json;

use super::profile_defaults::BEAVER_PROFILE_ID;
use super::profile_store::{
    forget_migration_marker_for_test, load_from_paths, save_to_path_fail_before_replace,
    trigger_settings, CompressionProfileStoreError,
};
use super::profile_store_document::{CompressionProfileDocument, PROFILE_SCHEMA_VERSION};

#[test]
fn missing_file_migrates_legacy_threshold_and_enabled_state() {
    let root = tempfile::tempdir().expect("temp root");
    let profile_path = root.path().join("compression-profiles.json");
    let config_path = root.path().join("config.json");
    write_config(&config_path, 85, false);

    let document = load_from_paths(&profile_path, &config_path).expect("migrate profiles");

    assert_eq!(document.schema_version, PROFILE_SCHEMA_VERSION);
    assert_eq!(document.global_profile_id, BEAVER_PROFILE_ID);
    assert_eq!(document.profiles[0].threshold_percent, 85);
    assert!(!document.automatic_enabled);
    let migrated: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&config_path).expect("config")).expect("json");
    assert!(migrated["advanced"].get("compression_enabled").is_none());
    assert!(migrated["advanced"].get("compression_threshold").is_none());
}

#[test]
fn legacy_thresholds_are_migrated_once_into_one_to_ninety() {
    for (legacy, expected) in [(0, 90), (85, 85), (95, 90)] {
        let root = tempfile::tempdir().expect("temp root");
        let profile_path = root.path().join("compression-profiles.json");
        let config_path = root.path().join("config.json");
        write_config(&config_path, legacy, true);
        let first = load_from_paths(&profile_path, &config_path).expect("first load");
        assert_eq!(first.profiles[0].threshold_percent, expected);

        write_config(&config_path, 12, true);
        let second = load_from_paths(&profile_path, &config_path).expect("second load");
        assert_eq!(second.profiles[0].threshold_percent, expected);
    }
}

#[test]
fn valid_v2_document_is_normalized_and_missing_global_falls_back() {
    let root = tempfile::tempdir().expect("temp root");
    let profile_path = root.path().join("compression-profiles.json");
    let config_path = root.path().join("config.json");
    let mut document = CompressionProfileDocument {
        global_profile_id: "missing".into(),
        ..CompressionProfileDocument::default()
    };
    document.profiles[0].threshold_percent = 200;
    std::fs::write(&profile_path, serde_json::to_vec(&document).expect("json")).expect("write");

    let loaded = load_from_paths(&profile_path, &config_path).expect("load");

    assert_eq!(loaded.global_profile_id, BEAVER_PROFILE_ID);
    assert_eq!(loaded.profiles[0].threshold_percent, 90);
}

#[test]
fn corrupt_json_recovers_to_the_bounded_default() {
    let root = tempfile::tempdir().expect("temp root");
    let profile_path = root.path().join("compression-profiles.json");
    let config_path = root.path().join("config.json");
    std::fs::write(&profile_path, b"{not-json").expect("write");

    let loaded = load_from_paths(&profile_path, &config_path).expect("recover");

    let expected = CompressionProfileDocument {
        recovery_backup_pending: true,
        ..CompressionProfileDocument::default()
    };
    assert_eq!(loaded, expected);
    let repaired: CompressionProfileDocument =
        serde_json::from_slice(&std::fs::read(&profile_path).expect("repaired profile document"))
            .expect("valid repaired json");
    assert_eq!(repaired, loaded);
}

#[test]
fn future_document_version_is_reported_without_rewriting_the_file() {
    let root = tempfile::tempdir().expect("temp root");
    let profile_path = root.path().join("compression-profiles.json");
    let config_path = root.path().join("config.json");
    let mut value = serde_json::to_value(CompressionProfileDocument::default()).expect("json");
    value["schema_version"] = serde_json::Value::from(PROFILE_SCHEMA_VERSION + 1);
    let original = serde_json::to_vec_pretty(&value).expect("json");
    std::fs::write(&profile_path, &original).expect("write");

    let error = load_from_paths(&profile_path, &config_path).expect_err("future version");

    assert_eq!(
        error,
        CompressionProfileStoreError::FutureVersion(PROFILE_SCHEMA_VERSION + 1)
    );
    assert_eq!(std::fs::read(&profile_path).expect("unchanged"), original);
}

#[test]
fn corrupt_unrelated_config_does_not_disable_the_profile_store() {
    let root = tempfile::tempdir().expect("temp root");
    let profile_path = root.path().join("compression-profiles.json");
    let config_path = root.path().join("config.json");
    std::fs::write(&config_path, b"{not-json").expect("write corrupt config");

    let loaded = load_from_paths(&profile_path, &config_path).expect("load profiles");

    assert_eq!(loaded, CompressionProfileDocument::default());
    assert_eq!(std::fs::read(&config_path).unwrap(), b"{not-json");
}

#[test]
fn missing_nested_fields_use_the_beaver_band_defaults() {
    let root = tempfile::tempdir().expect("temp root");
    let profile_path = root.path().join("compression-profiles.json");
    let config_path = root.path().join("config.json");
    let mut value = serde_json::to_value(CompressionProfileDocument::default()).expect("json");
    value["profiles"][0]["compact"]
        .as_object_mut()
        .expect("compact object")
        .remove("recent_message_count");
    std::fs::write(&profile_path, serde_json::to_vec(&value).expect("json")).expect("write");

    let loaded = load_from_paths(&profile_path, &config_path).expect("load");

    assert_eq!(loaded.profiles[0].compact.recent_message_count, 4);
}

#[test]
fn v1_profiles_migrate_identity_prompts_and_policy_but_reset_all_bands() {
    let root = tempfile::tempdir().expect("temp root");
    let profile_path = root.path().join("compression-profiles.json");
    let config_path = root.path().join("config.json");
    let custom_id = "00000000-0000-4000-8000-000000000001";
    let original = include_bytes!("fixtures/compression-profiles-v1.json").to_vec();
    std::fs::write(&profile_path, &original).expect("write legacy");

    let loaded = load_from_paths(&profile_path, &config_path).expect("migrate v1");

    assert_eq!(loaded.schema_version, 2);
    assert!(!loaded.automatic_enabled);
    assert_eq!(loaded.global_profile_id, custom_id);
    assert_eq!(loaded.global_selection_revision, 7);
    let custom = loaded
        .profiles
        .iter()
        .find(|profile| profile.id == custom_id)
        .unwrap();
    assert_eq!(custom.revision, 9);
    assert_eq!(custom.threshold_percent, 72);
    assert!(custom.allow_under_64k);
    assert_eq!(custom.system_prompt, "custom system");
    assert_eq!(custom.handoff_prompt, "custom handoff");
    assert_eq!(
        custom.compact,
        super::profile_defaults::beaver_profile().compact
    );
    assert_eq!(
        std::fs::read(root.path().join("compression-profiles.v1.bak")).expect("backup"),
        original
    );
}

#[test]
fn failed_v1_backup_keeps_the_source_document_untouched() {
    let root = tempfile::tempdir().expect("temp root");
    let profile_path = root.path().join("compression-profiles.json");
    let config_path = root.path().join("config.json");
    let backup_path = root.path().join("compression-profiles.v1.bak");
    let original = include_bytes!("fixtures/compression-profiles-v1.json").to_vec();
    std::fs::write(&profile_path, &original).expect("write legacy");
    std::fs::create_dir(&backup_path).expect("blocking backup directory");

    assert_eq!(
        load_from_paths(&profile_path, &config_path).unwrap_err(),
        CompressionProfileStoreError::Migration
    );
    assert_eq!(
        std::fs::read(&profile_path).expect("source remains"),
        original
    );
}

#[test]
fn repairing_an_unreadable_v2_document_keeps_the_v1_backup() {
    let root = tempfile::tempdir().expect("temp root");
    let profile_path = root.path().join("compression-profiles.json");
    let config_path = root.path().join("config.json");
    let backup_path = root.path().join("compression-profiles.v1.bak");
    let original = include_bytes!("fixtures/compression-profiles-v1.json").to_vec();
    std::fs::write(&profile_path, &original).expect("write legacy");
    load_from_paths(&profile_path, &config_path).expect("migrate legacy");
    assert_eq!(std::fs::read(&backup_path).expect("backup"), original);

    forget_migration_marker_for_test(&profile_path);
    std::fs::write(&profile_path, b"{not-json").expect("corrupt v2");
    load_from_paths(&profile_path, &config_path).expect("repair v2");

    assert_eq!(std::fs::read(&backup_path).expect("backup kept"), original);
    load_from_paths(&profile_path, &config_path).expect("reload repaired v2");
    assert_eq!(
        std::fs::read(&backup_path).expect("backup survives later reloads"),
        original
    );
}

#[test]
fn twenty_one_profiles_are_bounded_and_an_invalid_name_is_dropped() {
    let root = tempfile::tempdir().expect("temp root");
    let profile_path = root.path().join("compression-profiles.json");
    let config_path = root.path().join("config.json");
    let mut document = CompressionProfileDocument::default();
    for index in 0..20 {
        let mut profile = document.profiles[0].clone();
        profile.id = format!("00000000-0000-4000-8000-{index:012}");
        profile.name = format!("Custom {index}");
        document.profiles.push(profile);
    }
    document.profiles[1].name = "\u{0007}".into();
    std::fs::write(&profile_path, serde_json::to_vec(&document).expect("json")).expect("write");

    let loaded = load_from_paths(&profile_path, &config_path).expect("load");

    assert_eq!(loaded.profiles.len(), 20);
    assert!(loaded
        .profiles
        .iter()
        .all(|profile| !profile.name.chars().any(char::is_control)));
}

#[test]
fn failed_replace_keeps_the_previous_document() {
    let root = tempfile::tempdir().expect("temp root");
    let profile_path = root.path().join("compression-profiles.json");
    let original = CompressionProfileDocument::default();
    std::fs::write(&profile_path, serde_json::to_vec(&original).expect("json")).expect("write");
    let mut changed = original.clone();
    changed.profiles[0].threshold_percent = 10;

    assert!(save_to_path_fail_before_replace(&profile_path, &changed).is_err());
    let stored: CompressionProfileDocument =
        serde_json::from_slice(&std::fs::read(&profile_path).expect("stored original"))
            .expect("original json");
    assert_eq!(stored, original);
}

#[test]
fn trigger_adapter_uses_the_global_profile_and_under_64k_guard() {
    let mut document = CompressionProfileDocument::default();
    let tiny = trigger_settings(&document, 63_999).expect("tiny settings");
    assert_eq!(tiny.threshold_percent, 90);
    assert!(!tiny.available);
    assert!(
        trigger_settings(&document, 64_000)
            .expect("compact settings")
            .available
    );
    assert!(
        trigger_settings(&document, 0)
            .expect("unknown settings")
            .available
    );

    document.profiles[0].allow_under_64k = true;
    assert!(
        trigger_settings(&document, 32_000)
            .expect("enabled tiny settings")
            .available
    );

    document.automatic_enabled = false;
    assert!(
        !trigger_settings(&document, 128_000)
            .expect("disabled automatic compression")
            .available
    );
}

#[test]
fn migration_backup_survives_until_the_next_process_generation() {
    let root = tempfile::tempdir().expect("temp root");
    let profile_path = root.path().join("compression-profiles.json");
    let config_path = root.path().join("config.json");
    let backup_path = root.path().join("config.json.compression-v1.bak");
    write_config(&config_path, 85, true);

    load_from_paths(&profile_path, &config_path).expect("migrate");
    assert!(backup_path.exists());
    load_from_paths(&profile_path, &config_path).expect("same process reload");
    assert!(backup_path.exists());

    forget_migration_marker_for_test(&profile_path);
    load_from_paths(&profile_path, &config_path).expect("next process reload");
    assert!(!backup_path.exists());
}

fn write_config(path: &std::path::Path, threshold: u8, enabled: bool) {
    let config = json!({
        "advanced": {
            "compression_enabled": enabled,
            "compression_threshold": threshold
        }
    });
    std::fs::write(path, serde_json::to_vec_pretty(&config).expect("json")).expect("config");
}
