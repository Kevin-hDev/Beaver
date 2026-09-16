use super::store::{read_all_at, AUTOMATIONS_SCHEMA_VERSION};
use super::store_migration_v2::{acknowledge_successful_startup, load_or_migrate_stopping_after};
use crate::models::{
    AutomationDefinition, AutomationExtensionOwnership, AutomationSchedule, AutomationStatus,
    AutomationTarget,
};
use chrono::{TimeZone, Utc};
use serde_json::json;
use uuid::Uuid;

fn definition() -> AutomationDefinition {
    AutomationDefinition {
        id: Uuid::new_v4(),
        revision: 1,
        name: "Migrated".into(),
        description: None,
        prompt: "Run".into(),
        creator_session_id: None,
        target: AutomationTarget::NewSession { project_id: None },
        provider: "codex-oauth".into(),
        model: "gpt-test".into(),
        schedule: AutomationSchedule::Cron {
            expression: "0 8 * * *".into(),
            timezone: chrono_tz::UTC,
        },
        status: AutomationStatus::Disabled,
        created_at: Utc.with_ymd_and_hms(2026, 9, 1, 8, 0, 0).unwrap(),
        anchor_at: None,
        extension_owner: None,
    }
}

fn v1_bytes(owner: Option<serde_json::Value>) -> Vec<u8> {
    let file = super::store_wire::AutomationFile::from_definitions(vec![definition()]);
    let mut value = serde_json::to_value(file).unwrap();
    value["schema_version"] = json!(1);
    value["unknown_top"] = json!({"preserved": true});
    value["automations"][0]["unknown_definition"] = json!("kept-in-backup");
    if let Some(owner) = owner {
        value["automations"][0]["extension_owner"] = owner;
    }
    serde_json::to_vec_pretty(&value).unwrap()
}

#[tokio::test]
async fn automation_v1_migration_preserves_exact_backup_and_unknown_fields() {
    let root = tempfile::tempdir().unwrap();
    let original = v1_bytes(None);
    std::fs::write(root.path().join("automations.json"), &original).unwrap();
    let items = read_all_at(root.path()).await.unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        std::fs::read(root.path().join("automations.pre-v2.json")).unwrap(),
        original
    );
    let migrated: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.path().join("automations.json")).unwrap())
            .unwrap();
    assert_eq!(migrated["schema_version"], AUTOMATIONS_SCHEMA_VERSION);
}

#[tokio::test]
async fn future_automation_store_is_never_rewritten() {
    let root = tempfile::tempdir().unwrap();
    let mut value: serde_json::Value = serde_json::from_slice(&v1_bytes(None)).unwrap();
    value["schema_version"] = json!(AUTOMATIONS_SCHEMA_VERSION + 1);
    let bytes = serde_json::to_vec_pretty(&value).unwrap();
    let path = root.path().join("automations.json");
    std::fs::write(&path, &bytes).unwrap();
    assert_eq!(read_all_at(root.path()).await.unwrap().len(), 1);
    assert!(super::store::mutate_at(root.path(), |_| Ok(()))
        .await
        .is_err());
    assert_eq!(std::fs::read(path).unwrap(), bytes);
}

#[tokio::test]
async fn malformed_extension_owner_never_becomes_native() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(
        root.path().join("automations.json"),
        v1_bytes(Some(json!({"extensionId": "broken"}))),
    )
    .unwrap();
    let items = read_all_at(root.path()).await.unwrap();
    assert!(matches!(
        items[0].extension_owner,
        Some(AutomationExtensionOwnership::Invalid(_))
    ));
}

#[tokio::test]
async fn migration_recovers_after_each_v2_publication_boundary() {
    for boundary in [1, 2] {
        let root = tempfile::tempdir().unwrap();
        let bytes = v1_bytes(None);
        let path = root.path().join("automations.json");
        std::fs::write(&path, &bytes).unwrap();
        assert!(
            load_or_migrate_stopping_after(root.path(), path, bytes, Some(boundary),)
                .await
                .is_err()
        );
        assert_eq!(read_all_at(root.path()).await.unwrap().len(), 1);
        acknowledge_successful_startup(root.path()).unwrap();
        assert!(root.path().join("automations.pre-v2.json").exists());
        acknowledge_successful_startup(root.path()).unwrap();
        assert!(!root.path().join("automations.pre-v2.json").exists());
    }
}
