use super::storage;
use serde_json::Value;

const BUILTIN_V0: &[u8] =
    include_bytes!("../../../test-fixtures/extensions/extensions-v0-array.json");
const LOCAL_V0: &[u8] =
    include_bytes!("../../../test-fixtures/extensions/extensions-v0-array-with-local.json");
const REGISTRY_V1: &[u8] =
    include_bytes!("../../../test-fixtures/extensions/extensions-v1-envelope.json");
const LEGACY_UI_V1: &[u8] =
    include_bytes!("../../../test-fixtures/extensions/extensions-v1-legacy-ui.json");

fn write_local_fixture(source: &std::path::Path) -> Vec<u8> {
    let mut value: Value = serde_json::from_slice(LOCAL_V0).unwrap();
    value.as_array_mut().unwrap()[4]["source"] =
        Value::String(source.to_str().unwrap().to_string());
    serde_json::to_vec_pretty(&value).unwrap()
}

fn materialize_git_fixture(root: &std::path::Path) {
    std::fs::create_dir_all(root.join(".git")).unwrap();
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(root.join("index.mjs"), "export default {};\n").unwrap();
    std::fs::write(root.join("src/helper.ts"), "export const answer = 42;\n").unwrap();
    std::fs::write(
        root.join("beaver-extension.json"),
        r#"{"id":"com.example.git-fixture","name":"Git Fixture","version":"1.2.3","beaverApi":"1","runtime":"node","main":"index.mjs","ui":null,"access":"full","apiLevel":"stable","essential":false,"author":"Fixture Author","homepage":"https://example.invalid/fixture","description":"A migration fixture produced in the Beaver v1.2.0 record shape."}"#,
    )
    .unwrap();
}

fn persisted_v0_shape(records: &[super::types::ExtensionRecord]) -> Value {
    let mut value = serde_json::to_value(records).unwrap();
    for entry in value.as_array_mut().unwrap() {
        let object = entry.as_object_mut().unwrap();
        object.remove("fingerprint");
        object.remove("trustedAt");
        object.remove("sensitiveAccessGranted");
    }
    value
}

#[test]
fn real_v120_array_migrates_once_with_an_exact_backup_and_strict_v2_envelope() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("extensions.json");
    std::fs::write(&path, BUILTIN_V0).unwrap();

    let loaded = storage::load_from(&path).unwrap();

    assert_eq!(loaded.format, storage::LoadedFormat::MigratedV0);
    assert_eq!(loaded.extensions.len(), 4);
    assert_eq!(
        persisted_v0_shape(&loaded.extensions),
        serde_json::from_slice::<Value>(BUILTIN_V0).unwrap()
    );
    assert_eq!(
        std::fs::read(storage::v0_backup_path(&path)).unwrap(),
        BUILTIN_V0
    );
    let migrated: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(migrated["version"], 2);
    assert_eq!(migrated["extensions"].as_array().unwrap().len(), 4);
    assert!(migrated["recoverySnapshot"].is_null());

    let second = storage::load_from(&path).unwrap();
    assert_eq!(second.format, storage::LoadedFormat::V2);
    storage::finish_successful_startup(&path, second.format).unwrap();
    assert!(!storage::v0_backup_path(&path).exists());
}

#[test]
fn local_v0_record_keeps_its_fields_and_receives_a_current_fingerprint() {
    let directory = tempfile::tempdir().unwrap();
    let source = directory.path().join("source");
    std::fs::create_dir(&source).unwrap();
    materialize_git_fixture(&source);
    let path = directory.path().join("extensions.json");
    let fixture = write_local_fixture(&source);
    std::fs::write(&path, &fixture).unwrap();

    let loaded = storage::load_from(&path).unwrap();
    let local = loaded.extensions.last().unwrap();

    assert_eq!(local.manifest.id, "com.example.git-fixture");
    assert_eq!(local.manifest.name, "Git Fixture");
    assert_eq!(local.manifest.version, "1.2.3");
    assert_eq!(local.source, source.to_str().unwrap());
    assert_eq!(
        local.origin.as_ref().unwrap().locator,
        "https://example.com/example/fixture.git"
    );
    assert!(local.enabled);
    assert!(local.trusted);
    assert!(!local.show_in_chat);
    assert_eq!(local.status, super::types::ExtensionStatus::Error);
    assert_eq!(local.last_error.as_deref(), Some("previous_safe_error"));
    assert_eq!(
        local.last_activated_at.as_deref(),
        Some("2026-08-30T12:34:56Z")
    );
    assert_eq!(local.fingerprint.as_deref().map(str::len), Some(64));
    assert!(local.trusted_at.is_none());
    assert!(!local.sensitive_access_granted);
    assert_eq!(
        persisted_v0_shape(std::slice::from_ref(local))[0],
        serde_json::from_slice::<Value>(&fixture).unwrap()[4]
    );
}

#[test]
fn impossible_local_fingerprint_migrates_to_a_closed_record() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("extensions.json");
    let missing = directory.path().join("missing");
    std::fs::write(&path, write_local_fixture(&missing)).unwrap();

    let loaded = storage::load_from(&path).unwrap();
    let local = loaded.extensions.last().unwrap();

    assert!(!local.enabled);
    assert!(!local.trusted);
    assert_eq!(local.status, super::types::ExtensionStatus::Error);
    assert_eq!(
        local.last_error.as_deref(),
        Some("extensions_fingerprint_failed")
    );
    assert!(local.trusted_at.is_none());
    assert!(!local.sensitive_access_granted);
}

#[test]
fn unknown_external_entry_is_ignored_without_losing_supported_neighbors() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("extensions.json");
    let mut value: Value = serde_json::from_slice(BUILTIN_V0).unwrap();
    let mut external = value.as_array().unwrap()[0].clone();
    external["kind"] = Value::String("external".to_string());
    external["manifest"]["id"] = Value::String("com.example.external".to_string());
    value.as_array_mut().unwrap().insert(2, external);
    std::fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();

    let loaded = storage::load_from(&path).unwrap();

    assert_eq!(loaded.extensions.len(), 4);
    assert!(loaded
        .extensions
        .iter()
        .all(|record| record.manifest.id != "com.example.external"));
}

#[test]
fn oversized_v0_fails_before_backup_and_invalid_v1_keeps_an_existing_backup() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("extensions.json");
    std::fs::write(&path, vec![b' '; super::types::MAX_MESSAGE_BYTES + 1]).unwrap();

    assert_eq!(
        storage::load_from(&path).unwrap_err(),
        "extensions_registry_migration_failed"
    );
    assert!(!storage::v0_backup_path(&path).exists());

    std::fs::write(storage::v0_backup_path(&path), BUILTIN_V0).unwrap();
    std::fs::write(
        &path,
        br#"{"version":1,"extensions":"invalid","recoverySnapshot":null}"#,
    )
    .unwrap();
    assert_eq!(
        storage::load_from(&path).unwrap_err(),
        "extensions_registry_migration_failed"
    );
    assert!(storage::v0_backup_path(&path).exists());
}

#[test]
fn current_recovery_snapshot_is_preserved_and_bounded() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("extensions.json");
    let snapshot = Some(vec!["beaver.office.documents".to_string()]);

    storage::save_to(&path, &[], &snapshot).unwrap();
    let loaded = storage::load_from(&path).unwrap();
    assert_eq!(loaded.recovery_snapshot, snapshot);

    let oversized = Some(
        (0..=super::types::MAX_EXTENSIONS)
            .map(|index| format!("com.example.plugin-{index}"))
            .collect(),
    );
    assert_eq!(
        storage::save_to(&path, &[], &oversized).unwrap_err(),
        super::error_codes::RECOVERY_MARKER_INVALID
    );
}

#[test]
fn legacy_advanced_record_without_artifact_is_preserved_but_closed() {
    let directory = tempfile::tempdir().unwrap();
    let source = directory.path().join("advanced");
    std::fs::create_dir(&source).unwrap();
    std::fs::write(source.join("index.mjs"), "export default {};\n").unwrap();
    std::fs::write(source.join("ui.ts"), "export function activate() {}\n").unwrap();
    let mut record = super::manifest::load_local(source.join("index.mjs").to_str().unwrap())
        .unwrap()
        .record;
    record.manifest.ui = Some(super::types::ExtensionUiManifest {
        api_version: "1".to_string(),
        mode: super::types::ExtensionUiMode::Advanced,
        entry: Some("ui.ts".to_string()),
    });
    record.enabled = true;
    record.trusted = true;
    record.ui_artifact = None;

    let reset = super::registry_state::reset_hosted_runtime(vec![record]);

    assert!(!reset[0].enabled);
    assert!(!reset[0].trusted);
    assert_eq!(reset[0].status, super::types::ExtensionStatus::Error);
    assert_eq!(
        reset[0].last_error.as_deref(),
        Some(super::ui_contract::UI_DIAGNOSTIC_UI_ARTIFACT_INVALID),
    );
    assert!(super::validation::records(&reset).is_ok());
}

#[test]
fn v1_envelope_ignores_unknown_fields_for_forward_compatibility() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("extensions.json");
    std::fs::write(
        &path,
        br#"{"version":1,"extensions":[],"recoverySnapshot":null,"futureField":{"enabled":true}}"#,
    )
    .unwrap();

    let loaded = storage::load_from(&path).unwrap();
    assert!(loaded.extensions.is_empty());
    assert_eq!(loaded.format, storage::LoadedFormat::MigratedV1);
    let migrated: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(migrated["version"], 2);
    assert_eq!(migrated["futureField"]["enabled"], true);
}

#[test]
fn real_v1_registry_migrates_with_exact_backup_and_preserves_unknown_fields() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("extensions.json");
    let backup = path.with_extension("json.v1.bak");
    std::fs::write(&path, REGISTRY_V1).unwrap();

    let first = storage::load_from(&path).unwrap();
    assert_eq!(first.extensions.len(), 1);
    assert_eq!(std::fs::read(&backup).unwrap(), REGISTRY_V1);

    let migrated: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(migrated["version"], 2);
    assert_eq!(migrated["futureEnvelopeField"]["kept"], 42);
    assert_eq!(migrated["extensions"][0]["futureRecordField"][0], "kept");
    assert_eq!(
        migrated["extensions"][0]["manifest"]["futureManifestField"]["preserved"],
        true,
    );

    storage::save_to(&path, &first.extensions, &first.recovery_snapshot).unwrap();
    let saved: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(saved["futureEnvelopeField"]["kept"], 42);
    assert_eq!(saved["extensions"][0]["futureRecordField"][0], "kept");
    assert_eq!(
        saved["extensions"][0]["manifest"]["futureManifestField"]["preserved"],
        true,
    );

    storage::finish_successful_startup(&path, first.format).unwrap();
    assert!(backup.exists());
    let second = storage::load_from(&path).unwrap();
    storage::finish_successful_startup(&path, second.format).unwrap();
    assert!(!backup.exists());
}

#[test]
fn legacy_ui_string_is_retained_but_never_inferred_as_advanced() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("extensions.json");
    std::fs::write(&path, LEGACY_UI_V1).unwrap();

    let loaded = storage::load_from(&path).unwrap();
    assert_eq!(loaded.extensions.len(), 1);
    assert!(loaded.extensions[0].manifest.ui.is_none());
    assert_eq!(
        loaded.extensions[0].manifest.ui_legacy.as_deref(),
        Some("src/ui.tsx")
    );
    let migrated: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    let manifest = &migrated["extensions"][0]["manifest"];

    assert!(manifest["ui"].is_null());
    assert_eq!(manifest["uiLegacy"], "src/ui.tsx");
    assert_eq!(manifest["apiLevel"], "stable");

    storage::save_to(&path, &loaded.extensions, &loaded.recovery_snapshot).unwrap();
    let reloaded = storage::load_from(&path).unwrap();
    assert!(reloaded.extensions[0].manifest.ui.is_none());
    assert_eq!(
        reloaded.extensions[0].manifest.ui_legacy.as_deref(),
        Some("src/ui.tsx")
    );
}
