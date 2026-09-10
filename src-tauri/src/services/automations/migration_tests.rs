use super::migration::{
    acknowledge_successful_startup, migrate_legacy_at, migrate_legacy_stopping_after,
    AutomationMigrationStatus,
};
use super::migration_conflict::{resolve_at, ConflictResolution};
use super::runtime_store::{AutomationOccurrence, AutomationRuntime};
use chrono::{TimeZone, Utc};
use chrono_tz::Europe::Paris;

fn fixture_bytes() -> Vec<u8> {
    include_bytes!("fixtures/config-real-v1-sanitized.json").to_vec()
}

#[tokio::test]
async fn real_sanitized_config_migrates_without_losing_other_sections() {
    let root = tempfile::tempdir().unwrap();
    let fixture = include_bytes!("fixtures/config-real-v1-sanitized.json");
    std::fs::write(root.path().join("config.json"), fixture).unwrap();

    assert_eq!(
        migrate_legacy_at(root.path(), Some(Paris)).await.unwrap(),
        AutomationMigrationStatus::Ready
    );
    let config: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.path().join("config.json")).unwrap()).unwrap();
    assert!(config.get("scheduled_wakeups").is_none());
    assert_eq!(config["heartbeat"]["global_paused"], true);
    assert_eq!(config["gateway"]["audit"]["enabled"], true);
    assert_eq!(config["mascot"]["mascot_id"], "test-mascot");
    assert_eq!(config["future_section"]["kept"], true);
    assert!(root.path().join("config.pre-automations-v1.json").exists());

    let automations: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.path().join("automations.json")).unwrap())
            .unwrap();
    assert_eq!(automations["automations"][0]["target_mode"], "new_session");
    assert_eq!(automations["automations"][0]["schedule"]["kind"], "cron");
    assert_eq!(
        automations["automations"][0]["schedule"]["expression"],
        "0 8 * * *"
    );
}

#[tokio::test]
async fn missing_timezone_changes_nothing() {
    let root = tempfile::tempdir().unwrap();
    let fixture = include_bytes!("fixtures/config-real-v1-sanitized.json");
    std::fs::write(root.path().join("config.json"), fixture).unwrap();

    assert_eq!(
        migrate_legacy_at(root.path(), None).await.unwrap(),
        AutomationMigrationStatus::NeedsTimezone
    );
    assert_eq!(
        std::fs::read(root.path().join("config.json")).unwrap(),
        fixture
    );
    assert!(!root.path().join("automations.json").exists());
    assert!(!root.path().join("config.pre-automations-v1.json").exists());
}

#[tokio::test]
async fn migration_recovers_after_every_publication_boundary() {
    for boundary in 1..=5 {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("config.json"), fixture_bytes()).unwrap();
        assert!(
            migrate_legacy_stopping_after(root.path(), Some(Paris), boundary)
                .await
                .is_err()
        );
        assert_eq!(
            migrate_legacy_at(root.path(), Some(Paris)).await.unwrap(),
            AutomationMigrationStatus::Ready
        );
        assert_eq!(
            super::store::read_all_at(root.path()).await.unwrap().len(),
            1
        );
        let config: serde_json::Value =
            serde_json::from_slice(&std::fs::read(root.path().join("config.json")).unwrap())
                .unwrap();
        assert!(config.get("scheduled_wakeups").is_none());
    }
}

#[tokio::test]
async fn legacy_checkpoint_is_reused_and_cleanup_waits_for_a_successful_startup() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("config.json"), fixture_bytes()).unwrap();
    let checkpoint = Utc.with_ymd_and_hms(2026, 9, 9, 7, 0, 0).unwrap();
    std::fs::write(
        root.path().join("heartbeat-runtime.json"),
        format!(r#"{{"last_checked_at":"{}"}}"#, checkpoint.to_rfc3339()),
    )
    .unwrap();
    migrate_legacy_at(root.path(), Some(Paris)).await.unwrap();
    assert_eq!(
        super::runtime_store::read_at(root.path())
            .await
            .unwrap()
            .unwrap()
            .last_checked_at,
        checkpoint
    );
    assert!(root.path().join("config.pre-automations-v1.json").exists());
    acknowledge_successful_startup(root.path()).unwrap();
    migrate_legacy_at(root.path(), Some(Paris)).await.unwrap();
    assert!(root.path().join("config.pre-automations-v1.json").exists());
    acknowledge_successful_startup(root.path()).unwrap();
    assert!(!root.path().join("config.pre-automations-v1.json").exists());
    assert!(!root.path().join("heartbeat-runtime.json").exists());
}

#[tokio::test]
async fn old_version_reimport_is_idempotent_but_different_content_conflicts() {
    let root = tempfile::tempdir().unwrap();
    let original = fixture_bytes();
    std::fs::write(root.path().join("config.json"), &original).unwrap();
    migrate_legacy_at(root.path(), Some(Paris)).await.unwrap();

    std::fs::write(root.path().join("config.json"), &original).unwrap();
    assert_eq!(
        migrate_legacy_at(root.path(), Some(Paris)).await.unwrap(),
        AutomationMigrationStatus::Ready
    );
    assert_eq!(
        super::store::read_all_at(root.path()).await.unwrap().len(),
        1
    );

    let mut changed: serde_json::Value = serde_json::from_slice(&original).unwrap();
    changed["scheduled_wakeups"][0]["name"] = "Nom différent".into();
    std::fs::write(
        root.path().join("config.json"),
        serde_json::to_vec_pretty(&changed).unwrap(),
    )
    .unwrap();
    let status = migrate_legacy_at(root.path(), Some(Paris)).await.unwrap();
    assert!(matches!(status, AutomationMigrationStatus::Conflicts(items) if items.len() == 1));
    let remaining: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.path().join("config.json")).unwrap()).unwrap();
    assert_eq!(remaining["scheduled_wakeups"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn occurrence_reserves_a_deleted_definition_identifier() {
    let root = tempfile::tempdir().unwrap();
    let raw: serde_json::Value = serde_json::from_slice(&fixture_bytes()).unwrap();
    let automation_id = raw["scheduled_wakeups"][0]["id"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    let now = Utc::now();
    super::runtime_store::write_at(
        root.path(),
        &AutomationRuntime {
            schema_version: 1,
            last_checked_at: now,
            occurrences: vec![AutomationOccurrence::pending(automation_id, now)],
        },
    )
    .await
    .unwrap();
    std::fs::write(root.path().join("config.json"), fixture_bytes()).unwrap();

    let status = migrate_legacy_at(root.path(), Some(Paris)).await.unwrap();
    assert!(matches!(status, AutomationMigrationStatus::Conflicts(items) if items.len() == 1));
    assert!(super::store::read_all_at(root.path())
        .await
        .unwrap()
        .is_empty());
}

#[tokio::test]
async fn once_daily_and_weekly_keep_wall_time_and_active_state() {
    let root = tempfile::tempdir().unwrap();
    let mut config: serde_json::Value = serde_json::from_slice(&fixture_bytes()).unwrap();
    let template = config["scheduled_wakeups"][0].clone();
    let mut once = template.clone();
    once["id"] = uuid::Uuid::new_v4().to_string().into();
    once["schedule"] = serde_json::json!({"kind":"once","datetime":"2026-09-12T09:15"});
    once["active"] = true.into();
    let mut weekly = template;
    weekly["id"] = uuid::Uuid::new_v4().to_string().into();
    weekly["schedule"] = serde_json::json!({"kind":"weekly","weekday":3,"time":"17:45"});
    config["scheduled_wakeups"] = serde_json::json!([once, weekly]);
    std::fs::write(
        root.path().join("config.json"),
        serde_json::to_vec_pretty(&config).unwrap(),
    )
    .unwrap();

    migrate_legacy_at(root.path(), Some(Paris)).await.unwrap();
    let definitions = super::store::read_all_at(root.path()).await.unwrap();
    assert!(matches!(
        &definitions[0].schedule,
        crate::models::AutomationSchedule::Once { local_datetime, timezone }
            if local_datetime.to_string() == "2026-09-12 09:15:00" && *timezone == Paris
    ));
    assert_eq!(
        definitions[0].status,
        crate::models::AutomationStatus::Active
    );
    assert!(matches!(
        &definitions[1].schedule,
        crate::models::AutomationSchedule::Cron { expression, timezone }
            if expression == "45 17 * * 3" && *timezone == Paris
    ));
    assert_eq!(
        definitions[1].status,
        crate::models::AutomationStatus::Disabled
    );
}

#[tokio::test]
async fn conflict_can_be_removed_or_imported_under_a_backend_uuid() {
    for import in [false, true] {
        let root = tempfile::tempdir().unwrap();
        let original = fixture_bytes();
        std::fs::write(root.path().join("config.json"), &original).unwrap();
        migrate_legacy_at(root.path(), Some(Paris)).await.unwrap();
        let legacy_id = "44ddd259-54a5-42ae-81b5-5eb3ccee6ace";
        let original_id: uuid::Uuid = legacy_id.parse().unwrap();
        let mut changed: serde_json::Value = serde_json::from_slice(&original).unwrap();
        changed["scheduled_wakeups"][0]["name"] = "Copie historique".into();
        std::fs::write(
            root.path().join("config.json"),
            serde_json::to_vec_pretty(&changed).unwrap(),
        )
        .unwrap();
        assert!(matches!(
            migrate_legacy_at(root.path(), Some(Paris)).await.unwrap(),
            AutomationMigrationStatus::Conflicts(_)
        ));

        let new_id = resolve_at(
            root.path(),
            legacy_id,
            if import {
                ConflictResolution::ImportAsNew { timezone: Paris }
            } else {
                ConflictResolution::RemoveHistorical
            },
        )
        .await
        .unwrap();
        let definitions = super::store::read_all_at(root.path()).await.unwrap();
        assert_eq!(definitions.len(), if import { 2 } else { 1 });
        assert_eq!(new_id.is_some(), import);
        assert!(new_id.map(|id| id != original_id).unwrap_or(true));
        let config: serde_json::Value =
            serde_json::from_slice(&std::fs::read(root.path().join("config.json")).unwrap())
                .unwrap();
        assert!(config.get("scheduled_wakeups").is_none());
        assert!(!root
            .path()
            .join(".automation-conflict-resolution.json")
            .exists());
    }
}
