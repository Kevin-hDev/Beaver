use super::*;
use crate::models::{AutomationSchedule, AutomationStatus};
use serde_json::json;

static TEST_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

#[test]
fn parses_the_three_schedule_kinds_and_stable_errors() {
    assert!(matches!(
        validation::schedule(ScheduleInput::AfterCompletion { delay_minutes: 10 }).unwrap(),
        AutomationSchedule::AfterCompletion { delay_minutes: 10 }
    ));
    assert!(matches!(
        validation::schedule(ScheduleInput::Cron {
            expression: "*/10 * * * *".into(),
            timezone: "Europe/Paris".into(),
        })
        .unwrap(),
        AutomationSchedule::Cron { .. }
    ));
    assert!(matches!(
        validation::schedule(ScheduleInput::Once {
            local_datetime: "2026-09-12T09:30".into(),
            timezone: "Europe/Paris".into(),
        })
        .unwrap(),
        AutomationSchedule::Once { .. }
    ));
    assert!(validation::schedule(ScheduleInput::Cron {
        expression: "not cron".into(),
        timezone: "Europe/Paris".into(),
    })
    .is_err());
    assert_eq!(
        command_error(crate::services::automations::AutomationError::AuditUnavailable),
        "audit_unavailable"
    );
}

#[tokio::test]
async fn ipc_helpers_cover_crud_history_and_global_pause() {
    let _guard = TEST_LOCK.lock().await;
    crate::services::automations::mutate(|items| {
        items.clear();
        Ok(())
    })
    .await
    .unwrap();
    set_global_paused_inner(false).unwrap();

    let created = create_wakeup_inner(CreateWakeupInput {
        name: "CI".into(),
        model: "gpt-5.6-luna".into(),
        provider: "codex-oauth".into(),
        prompt: "Vérifie la CI".into(),
        schedule: ScheduleInput::AfterCompletion { delay_minutes: 10 },
        description: Some("Surveillance".into()),
        project_id: None,
    })
    .await
    .unwrap();
    let id = created.definition.id;
    assert_eq!(list_wakeups_inner().await.unwrap().len(), 1);
    assert_eq!(get_wakeup_inner(id).await.unwrap().definition.id, id);
    let updated = update_wakeup_inner(UpdateWakeupInput {
        automation_id: id,
        name: Some("CI verte".into()),
        description: None,
        prompt: None,
        model: None,
        schedule: None,
        status: Some(AutomationStatus::Disabled),
    })
    .await
    .unwrap();
    assert_eq!(updated.definition.name, "CI verte");
    assert_eq!(updated.definition.status, AutomationStatus::Disabled);
    assert!(list_wakeup_runs_inner(id, Some(20), None)
        .await
        .unwrap()
        .entries
        .is_empty());

    let before_pause = crate::services::automations::read_all().await.unwrap();
    set_global_paused_inner(true).unwrap();
    assert_eq!(
        crate::services::automations::read_all().await.unwrap(),
        before_pause
    );
    delete_wakeup_inner(id).await.unwrap();
    assert!(crate::services::automations::read_all()
        .await
        .unwrap()
        .is_empty());
    set_global_paused_inner(false).unwrap();
}

#[tokio::test]
async fn migration_waits_for_timezone_then_resolves_both_conflict_choices() {
    let root = tempfile::tempdir().unwrap();
    let first_id = uuid::Uuid::new_v4();
    write_legacy(root.path(), first_id, "Première");
    assert_eq!(
        migration_status_at(root.path(), None).await.unwrap().status,
        "needs_timezone"
    );
    assert_eq!(
        migration_status_at(root.path(), Some("Europe/Paris"))
            .await
            .unwrap()
            .status,
        "ready"
    );

    write_legacy(root.path(), first_id, "Collision à retirer");
    let conflicts = migration_status_at(root.path(), Some("Europe/Paris"))
        .await
        .unwrap();
    assert_eq!(conflicts.conflicts.len(), 1);
    assert!(resolve_conflict_at(
        root.path(),
        first_id.to_string(),
        ConflictDecision::RemoveHistorical,
        None,
    )
    .await
    .unwrap()
    .is_none());

    write_legacy(root.path(), first_id, "Collision à importer");
    migration_status_at(root.path(), Some("Europe/Paris"))
        .await
        .unwrap();
    let imported = resolve_conflict_at(
        root.path(),
        first_id.to_string(),
        ConflictDecision::ImportAsNew,
        Some("Europe/Paris".into()),
    )
    .await
    .unwrap()
    .unwrap();
    assert_ne!(imported, first_id);
    let audit = std::fs::read_to_string(root.path().join("logs/automation-audit.jsonl")).unwrap();
    assert!(audit.contains("resolve_migration_conflict"));
}

fn write_legacy(root: &std::path::Path, id: uuid::Uuid, name: &str) {
    std::fs::write(
        root.join("config.json"),
        serde_json::to_vec_pretty(&json!({
            "heartbeat":{"global_paused":false},
            "scheduled_wakeups":[{
                "id":id,
                "name":name,
                "model":"gpt-5.6-luna",
                "provider":"codex-oauth",
                "prompt":"Vérifie",
                "schedule":{"kind":"daily","time":"08:00"},
                "description":"",
                "active":true,
                "paused_by_global":false,
                "created_at":"2026-09-10T08:00:00Z"
            }]
        }))
        .unwrap(),
    )
    .unwrap();
}
