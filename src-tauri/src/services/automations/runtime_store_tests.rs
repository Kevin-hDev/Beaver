use super::reserved_automation_ids_at;
use super::runtime_store::{
    read_at, recover_startup_at, scan_and_advance_at, AutomationOccurrence, AutomationRuntime,
    OccurrenceState,
};
use chrono::{TimeZone, Utc};
use uuid::Uuid;

fn cron_definition(
    id: Uuid,
    created_at: chrono::DateTime<Utc>,
) -> crate::models::AutomationDefinition {
    crate::models::AutomationDefinition {
        id,
        revision: 1,
        name: "Cron".into(),
        description: None,
        prompt: "Vérifie".into(),
        creator_session_id: None,
        target: crate::models::AutomationTarget::NewSession { project_id: None },
        provider: "codex-oauth".into(),
        model: "gpt-5.6-luna".into(),
        schedule: crate::models::AutomationSchedule::Cron {
            expression: "* * * * *".into(),
            timezone: chrono_tz::UTC,
        },
        status: crate::models::AutomationStatus::Active,
        created_at,
        anchor_at: None,
    }
}

#[tokio::test]
async fn first_start_sets_checkpoint_without_inventing_occurrences() {
    let root = tempfile::tempdir().unwrap();
    let now = Utc.with_ymd_and_hms(2026, 9, 10, 10, 0, 0).unwrap();

    assert!(recover_startup_at(root.path(), now)
        .await
        .unwrap()
        .is_empty());
    let runtime = read_at(root.path()).await.unwrap().unwrap();
    assert_eq!(runtime.last_checked_at, now);
    assert!(runtime.occurrences.is_empty());
}

#[tokio::test]
async fn scan_persists_occurrence_and_checkpoint_together() {
    let root = tempfile::tempdir().unwrap();
    let start = Utc.with_ymd_and_hms(2026, 9, 10, 10, 0, 0).unwrap();
    super::runtime_store::write_at(
        root.path(),
        &AutomationRuntime {
            schema_version: 1,
            last_checked_at: start,
            occurrences: Vec::new(),
            retired_automation_ids: Vec::new(),
        },
    )
    .await
    .unwrap();
    let through = start + chrono::Duration::minutes(1);
    let automation_id = Uuid::new_v4();
    let occurrence = AutomationOccurrence::pending(automation_id, through);

    scan_and_advance_at(root.path(), through, |_| Ok(vec![occurrence.clone()]))
        .await
        .unwrap();
    let runtime = read_at(root.path()).await.unwrap().unwrap();
    assert_eq!(runtime.last_checked_at, through);
    assert_eq!(runtime.occurrences, vec![occurrence]);
    assert_eq!(
        reserved_automation_ids_at(root.path()).await.unwrap(),
        [automation_id].into_iter().collect()
    );
}

#[tokio::test]
async fn recovery_terminalizes_running_and_pending_and_is_idempotent() {
    let root = tempfile::tempdir().unwrap();
    let now = Utc.with_ymd_and_hms(2026, 9, 10, 12, 0, 0).unwrap();
    let mut pending = AutomationOccurrence::pending(Uuid::new_v4(), now);
    pending.coalesced_count = Some(3);
    pending.last_scheduled_for = Some(now + chrono::Duration::minutes(2));
    let mut running = AutomationOccurrence::pending(Uuid::new_v4(), now);
    running.state = OccurrenceState::Running;
    running.started_at = Some(now);
    running.coalesced_count = None;
    running.last_scheduled_for = None;
    super::runtime_store::write_at(
        root.path(),
        &AutomationRuntime {
            schema_version: 1,
            last_checked_at: now,
            occurrences: vec![pending, running],
            retired_automation_ids: Vec::new(),
        },
    )
    .await
    .unwrap();

    let first = recover_startup_at(root.path(), now).await.unwrap();
    let second = recover_startup_at(root.path(), now).await.unwrap();
    assert_eq!(first.len(), 2);
    assert_eq!(second, first);
    assert!(read_at(root.path())
        .await
        .unwrap()
        .unwrap()
        .occurrences
        .iter()
        .all(|item| item.state == OccurrenceState::Terminal));
}

#[tokio::test]
async fn runtime_rejects_more_than_128_occurrences() {
    let root = tempfile::tempdir().unwrap();
    let now = Utc::now();
    let occurrences = (0..129)
        .map(|_| AutomationOccurrence::pending(Uuid::new_v4(), now))
        .collect();
    assert!(super::runtime_store::write_at(
        root.path(),
        &AutomationRuntime {
            schema_version: 1,
            last_checked_at: now,
            occurrences,
            retired_automation_ids: Vec::new(),
        }
    )
    .await
    .is_err());
}

#[tokio::test]
async fn runtime_rejects_two_pending_occurrences_for_one_automation() {
    let root = tempfile::tempdir().unwrap();
    let now = Utc::now();
    let automation_id = Uuid::new_v4();
    let runtime = AutomationRuntime {
        schema_version: 1,
        last_checked_at: now,
        occurrences: vec![
            AutomationOccurrence::pending(automation_id, now),
            AutomationOccurrence::pending(automation_id, now),
        ],
        retired_automation_ids: Vec::new(),
    };
    assert!(super::runtime_store::write_at(root.path(), &runtime)
        .await
        .is_err());
}

#[tokio::test]
async fn runtime_wire_shape_depends_on_the_occurrence_state() {
    let root = tempfile::tempdir().unwrap();
    let now = Utc::now();
    let pending = AutomationOccurrence::pending(Uuid::new_v4(), now);
    super::runtime_store::write_at(
        root.path(),
        &AutomationRuntime {
            schema_version: 1,
            last_checked_at: now,
            occurrences: vec![pending],
            retired_automation_ids: Vec::new(),
        },
    )
    .await
    .unwrap();
    let path = root.path().join("automation-runtime.json");
    let pending: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    assert!(pending["occurrences"][0].get("coalesced_count").is_some());
    assert!(pending["occurrences"][0].get("started_at").is_none());
    assert!(pending["occurrences"][0].get("result").is_none());

    recover_startup_at(root.path(), now).await.unwrap();
    let terminal: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    assert!(terminal["occurrences"][0].get("coalesced_count").is_none());
    assert!(terminal["occurrences"][0]
        .get("started_at")
        .unwrap()
        .is_null());
    assert_eq!(terminal["occurrences"][0]["result"]["status"], "missed");
    assert!(terminal["occurrences"][0]["result"].get("tokens").is_some());
}

#[tokio::test]
async fn future_runtime_version_is_rejected_without_rewrite() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("automation-runtime.json");
    let future =
        br#"{"schema_version":2,"last_checked_at":"2026-09-10T00:00:00Z","occurrences":[]}"#;
    std::fs::write(&path, future).unwrap();
    assert!(recover_startup_at(root.path(), Utc::now()).await.is_err());
    assert_eq!(std::fs::read(path).unwrap(), future);
}

#[tokio::test]
async fn retry_before_or_after_scan_does_not_lose_or_duplicate_an_occurrence() {
    let root = tempfile::tempdir().unwrap();
    let start = Utc.with_ymd_and_hms(2026, 9, 10, 10, 0, 0).unwrap();
    let through = start + chrono::Duration::minutes(1);
    super::runtime_store::write_at(
        root.path(),
        &AutomationRuntime {
            schema_version: 1,
            last_checked_at: start,
            occurrences: Vec::new(),
            retired_automation_ids: Vec::new(),
        },
    )
    .await
    .unwrap();
    let definitions = vec![cron_definition(Uuid::new_v4(), start)];

    let first = super::runtime_scan::scan_definitions_at(root.path(), through, &definitions)
        .await
        .unwrap();
    let retry = super::runtime_scan::scan_definitions_at(root.path(), through, &definitions)
        .await
        .unwrap();
    let runtime = super::runtime_store::read_at(root.path())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(first.len(), 1);
    assert!(retry.is_empty());
    assert_eq!(runtime.last_checked_at, through);
    assert_eq!(runtime.occurrences.len(), 1);
}

#[tokio::test]
async fn overdue_ticks_merge_behind_a_running_occurrence_instead_of_becoming_missed() {
    let root = tempfile::tempdir().unwrap();
    let start = Utc.with_ymd_and_hms(2026, 9, 10, 10, 0, 0).unwrap();
    let automation_id = Uuid::new_v4();
    let mut running = AutomationOccurrence::pending(automation_id, start);
    running.state = OccurrenceState::Running;
    running.started_at = Some(start);
    running.coalesced_count = None;
    running.last_scheduled_for = None;
    super::runtime_store::write_at(
        root.path(),
        &AutomationRuntime {
            schema_version: 1,
            last_checked_at: start,
            occurrences: vec![running],
            retired_automation_ids: Vec::new(),
        },
    )
    .await
    .unwrap();

    super::runtime_scan::scan_definitions_at(
        root.path(),
        start + chrono::Duration::minutes(20),
        &[cron_definition(automation_id, start)],
    )
    .await
    .unwrap();
    let runtime = super::runtime_store::read_at(root.path())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(runtime.occurrences.len(), 2);
    let pending = runtime
        .occurrences
        .iter()
        .find(|item| item.state == OccurrenceState::Pending)
        .unwrap();
    assert_eq!(pending.coalesced_count, Some(20));
    assert!(!runtime.occurrences.iter().any(|item| {
        item.state == OccurrenceState::Terminal
            && item.result.as_ref().map(|result| &result.status)
                == Some(&super::runtime_store::OccurrenceResultStatus::Missed)
    }));
}
