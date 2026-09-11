use crate::services::scheduler;
use chrono::Utc;
use serde_json::{json, Value};
use uuid::Uuid;

#[tokio::test]
async fn deletion_and_legacy_reconciliation_are_serialized_in_both_orders() {
    for publish_first in [false, true] {
        cross_race(publish_first).await;
    }
}

#[tokio::test]
async fn full_retirement_registry_blocks_before_definition_deletion() {
    let root = tempfile::tempdir().unwrap();
    let fixture = include_bytes!("fixtures/config-real-v1-sanitized.json");
    std::fs::write(root.path().join("config.json"), fixture).unwrap();
    super::migration::migrate_legacy_at(root.path(), Some(chrono_tz::Europe::Paris))
        .await
        .unwrap();
    let definition = super::read_automations_at(root.path()).await.unwrap()[0].clone();
    let now = Utc::now();
    let mut occurrence = super::AutomationOccurrence::pending(definition.id, now);
    occurrence.state = super::OccurrenceState::Running;
    occurrence.coalesced_count = None;
    occurrence.last_scheduled_for = None;
    occurrence.started_at = Some(now);
    super::runtime_store::write_at(
        root.path(),
        &super::AutomationRuntime {
            schema_version: 1,
            last_checked_at: now,
            occurrences: vec![occurrence],
            retired_automation_ids: (0..128).map(|_| Uuid::new_v4()).collect(),
        },
    )
    .await
    .unwrap();

    assert_eq!(
        super::delete_at(root.path(), &ui_actor(), definition.id).await,
        Err(super::AutomationError::CapacityReached)
    );
    assert!(super::read_automations_at(root.path())
        .await
        .unwrap()
        .iter()
        .any(|item| item.id == definition.id));
}

async fn cross_race(publish_first: bool) {
    let root = tempfile::tempdir().unwrap();
    let fixture = include_bytes!("fixtures/config-real-v1-sanitized.json");
    std::fs::write(root.path().join("config.json"), fixture).unwrap();
    super::migration::migrate_legacy_at(root.path(), Some(chrono_tz::Europe::Paris))
        .await
        .unwrap();
    let original = super::read_automations_at(root.path()).await.unwrap()[0].clone();
    let now = Utc::now();
    let occurrence = super::AutomationOccurrence::pending(original.id, now);
    let occurrence_id = occurrence.id;
    super::runtime_store::write_at(
        root.path(),
        &super::AutomationRuntime {
            schema_version: 1,
            last_checked_at: now,
            occurrences: vec![occurrence],
            retired_automation_ids: Vec::new(),
        },
    )
    .await
    .unwrap();
    super::mark_runtime_running_at(root.path(), occurrence_id, now)
        .await
        .unwrap();
    super::mark_runtime_terminal_at(
        root.path(),
        occurrence_id,
        super::automation_e2e_tests::run_result("old-session", now),
    )
    .await
    .unwrap();
    super::delete_at(root.path(), &ui_actor(), original.id)
        .await
        .unwrap();

    let mut config: Value = serde_json::from_slice(fixture).unwrap();
    let historical = config["scheduled_wakeups"][0].clone();
    let other_id = Uuid::new_v4();
    let mut other = historical.clone();
    other["id"] = other_id.to_string().into();
    other["name"] = "Autre migration".into();
    config["scheduled_wakeups"] = json!([historical, other]);
    std::fs::write(
        root.path().join("config.json"),
        serde_json::to_vec_pretty(&config).unwrap(),
    )
    .unwrap();

    // Le verrou déjà pris sert de barrière et Tokio distribue ses attentes en FIFO.
    let barrier = super::store_lock().await;
    let root_path = root.path().to_path_buf();
    let first = spawn_ordered(root_path, occurrence_id, publish_first);
    tokio::task::yield_now().await;
    let root_path = root.path().to_path_buf();
    let second = spawn_ordered(root_path, occurrence_id, !publish_first);
    tokio::task::yield_now().await;
    drop(barrier);
    let migration_status = first
        .await
        .unwrap()
        .unwrap()
        .or(second.await.unwrap().unwrap())
        .unwrap();

    assert!(matches!(
        migration_status,
        super::migration::AutomationMigrationStatus::Conflicts(_)
    ));
    let definitions = super::read_automations_at(root.path()).await.unwrap();
    assert!(definitions.iter().all(|item| item.id != original.id));
    assert!(definitions.iter().any(|item| item.id == other_id));
    assert!(definitions.iter().all(|item| item.anchor_at.is_none()));
    let remaining: Value =
        serde_json::from_slice(&std::fs::read(root.path().join("config.json")).unwrap()).unwrap();
    assert_eq!(remaining["scheduled_wakeups"].as_array().unwrap().len(), 1);

    super::migration_conflict::resolve_at(
        root.path(),
        &original.id.to_string(),
        super::migration_conflict::ConflictResolution::RemoveHistorical,
    )
    .await
    .unwrap();
    assert!(!super::reserved_automation_ids_at(root.path())
        .await
        .unwrap()
        .contains(&original.id));
}

fn spawn_ordered(
    root: std::path::PathBuf,
    occurrence_id: Uuid,
    publish: bool,
) -> tokio::task::JoinHandle<
    Result<Option<super::migration::AutomationMigrationStatus>, super::AutomationError>,
> {
    tokio::spawn(async move {
        if publish {
            scheduler::publish_terminal_for_test(&root, occurrence_id)
                .await
                .map(|_| None)
        } else {
            super::migration::migrate_legacy_at(&root, Some(chrono_tz::Europe::Paris))
                .await
                .map(Some)
                .map_err(|_| super::AutomationError::StoreUnavailable)
        }
    })
}

fn ui_actor() -> super::AutomationActor {
    super::AutomationActor {
        origin: super::AutomationOrigin::UserInterface,
        session_or_channel_id: "automation-race".into(),
        current_automation_id: None,
    }
}
