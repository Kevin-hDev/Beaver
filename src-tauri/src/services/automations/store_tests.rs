use super::store::{mutate_at, read_all_at, write_file_at};
use super::store_wire::AutomationFile;
use crate::models::{AutomationDefinition, AutomationSchedule, AutomationStatus, AutomationTarget};
use chrono::{NaiveDate, TimeZone, Utc};
use uuid::Uuid;

fn definition(id: Uuid) -> AutomationDefinition {
    AutomationDefinition {
        id,
        revision: 1,
        name: "CI".into(),
        description: None,
        prompt: "Vérifie la CI".into(),
        creator_session_id: None,
        target: AutomationTarget::NewSession { project_id: None },
        provider: "codex-oauth".into(),
        model: "gpt-5.6-luna".into(),
        schedule: AutomationSchedule::Once {
            local_datetime: NaiveDate::from_ymd_opt(2026, 9, 11)
                .unwrap()
                .and_hms_opt(8, 0, 0)
                .unwrap(),
            timezone: chrono_tz::Europe::Paris,
        },
        status: AutomationStatus::Active,
        created_at: Utc.with_ymd_and_hms(2026, 9, 10, 8, 0, 0).unwrap(),
        anchor_at: None,
    }
}

#[tokio::test]
async fn missing_store_is_empty_and_v1_round_trips() {
    let root = tempfile::tempdir().unwrap();
    assert!(read_all_at(root.path()).await.unwrap().is_empty());

    let expected = definition(Uuid::new_v4());
    mutate_at(root.path(), |items| {
        items.push(expected.clone());
        Ok(())
    })
    .await
    .unwrap();

    assert_eq!(read_all_at(root.path()).await.unwrap(), vec![expected]);
    let raw: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.path().join("automations.json")).unwrap())
            .unwrap();
    assert_eq!(raw["schema_version"], 1);
    assert_eq!(raw["automations"][0]["target_mode"], "new_session");
    assert!(raw["automations"][0].get("target").is_none());
}

#[tokio::test]
async fn future_version_and_capacity_are_rejected_without_rewrite() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("automations.json");
    let future = br#"{"schema_version":2,"automations":[]}"#;
    std::fs::write(&path, future).unwrap();
    assert!(mutate_at(root.path(), |_| Ok(())).await.is_err());
    assert_eq!(std::fs::read(&path).unwrap(), future);

    let items = (0..65).map(|_| definition(Uuid::new_v4())).collect();
    assert!(
        write_file_at(root.path(), &AutomationFile::from_definitions(items))
            .await
            .is_err()
    );
    assert_eq!(std::fs::read(&path).unwrap(), future);
}

#[test]
fn atomic_publication_leaves_a_complete_old_or_new_document() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("automations.json");
    let old = br#"{"schema_version":1,"automations":[]}"#;
    let new = br#"{"schema_version":1,"automations":[{"id":"new"}]}"#;
    std::fs::write(&path, old).unwrap();

    assert!(crate::services::private_store::atomic_write_fail_before_replace(&path, new).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), old);
    crate::services::private_store::atomic_write(&path, new).unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), new);
    assert!(serde_json::from_slice::<serde_json::Value>(new).is_ok());
}
