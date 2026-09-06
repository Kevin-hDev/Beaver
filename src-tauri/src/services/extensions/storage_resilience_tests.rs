use super::storage;

#[test]
fn future_format_is_identified_before_parsing_its_unknown_schema() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("extensions.json");
    let bytes = br#"{"version":256,"extensions":{"newSchema":true}}"#;
    std::fs::write(&path, bytes).unwrap();
    assert_eq!(
        storage::load_from(&path).unwrap_err(),
        "extensions_registry_version_unsupported"
    );
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
}

#[test]
fn refused_registry_and_existing_backups_are_never_overwritten() {
    for bytes in [
        br#"{"version":3,"extensions":[]}"#.as_slice(),
        b"{broken",
        br#"{"version":2,"extensions":[{"kind":"local"}]}"#,
    ] {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("extensions.json");
        std::fs::write(&path, bytes).unwrap();
        let backup = storage::v1_backup_path(&path);
        std::fs::write(&backup, b"original backup").unwrap();
        assert!(storage::load_from(&path).is_err());
        assert!(storage::save_to(&path, &[], &None).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        assert_eq!(std::fs::read(&backup).unwrap(), b"original backup");
    }
}

#[test]
fn unreadable_registry_is_distinct_from_invalid_json() {
    let directory = tempfile::tempdir().unwrap();
    assert_eq!(
        storage::load_from(directory.path()).unwrap_err(),
        "extensions_registry_unavailable"
    );
    let path = directory.path().join("extensions.json");
    std::fs::write(&path, b"{").unwrap();
    assert_eq!(
        storage::load_from(&path).unwrap_err(),
        super::error_codes::REGISTRY_MIGRATION_FAILED
    );
}

#[test]
fn invalid_legacy_manifest_is_refused_before_migration_writes() {
    let fixture = include_bytes!("../../../test-fixtures/extensions/extensions-v1-envelope.json");
    let mut value: serde_json::Value = serde_json::from_slice(fixture).unwrap();
    value["extensions"][0]["manifest"]["id"] = serde_json::json!("invalid/id");
    let bytes = serde_json::to_vec(&value).unwrap();
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("extensions.json");
    std::fs::write(&path, &bytes).unwrap();
    assert!(storage::load_from(&path).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    assert!(!storage::v1_backup_path(&path).exists());
}

#[test]
fn clearing_legacy_ui_does_not_resurrect_the_persisted_value() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("extensions.json");
    let mut record = super::builtin::records().unwrap().remove(0);
    record.manifest.ui = None;
    record.manifest.ui_legacy = Some("legacy.ts".to_string());
    storage::save_to(&path, &[record.clone()], &None).unwrap();
    record.manifest.ui_legacy = None;
    storage::save_to(&path, &[record], &None).unwrap();
    assert!(storage::load_from(&path).unwrap().extensions[0]
        .manifest
        .ui_legacy
        .is_none());
}
