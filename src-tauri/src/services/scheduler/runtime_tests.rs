use super::runtime::{scan_if_active_at, sleep_until_next, terminal_ids};
use super::runtime_test_support::{
    mark_running_at, mark_terminal_at, publish_terminal_at, runtime_at, AutomationRunResult,
};
use crate::models::{AutomationDefinition, AutomationSchedule, AutomationStatus, AutomationTarget};
use crate::services::automations::{
    all_history_at, append_history_at, mutate_automations_at, read_automations_at,
    recover_startup_at, HistoryEntry, OccurrenceResultStatus, OccurrenceState,
};
use chrono::{TimeZone, Utc};
use uuid::Uuid;

fn definition(id: Uuid) -> AutomationDefinition {
    let created_at = Utc.with_ymd_and_hms(2026, 9, 10, 9, 0, 0).unwrap();
    AutomationDefinition {
        id,
        revision: 1,
        name: "CI".into(),
        description: None,
        prompt: "Vérifie la CI".into(),
        creator_session_id: Some("session-a".into()),
        target: AutomationTarget::ResumeSession {
            session_id: "session-a".into(),
        },
        provider: "codex-oauth".into(),
        model: "gpt-5.6-luna".into(),
        schedule: AutomationSchedule::Cron {
            expression: "*/5 * * * *".into(),
            timezone: chrono_tz::UTC,
        },
        status: AutomationStatus::Active,
        created_at,
        anchor_at: None,
    }
}

fn at(minute: u32) -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 9, 10, 10, minute, 0).unwrap()
}

fn once_definition(id: Uuid, scheduled_for: chrono::DateTime<Utc>) -> AutomationDefinition {
    let mut automation = definition(id);
    automation.schedule = AutomationSchedule::Once {
        local_datetime: scheduled_for.naive_utc(),
        timezone: chrono_tz::UTC,
    };
    automation
}

async fn scan_from(
    root: &std::path::Path,
    checkpoint: chrono::DateTime<Utc>,
    through: chrono::DateTime<Utc>,
    definitions: &[AutomationDefinition],
) -> Vec<Uuid> {
    scan_if_active_at(root, false, checkpoint, &[])
        .await
        .unwrap();
    scan_if_active_at(root, false, through, definitions)
        .await
        .unwrap()
}

#[tokio::test]
async fn grace_is_ready_but_an_old_unclaimed_occurrence_is_missed() {
    let ready_root = tempfile::tempdir().unwrap();
    let ready = once_definition(Uuid::new_v4(), at(0));
    scan_from(
        ready_root.path(),
        at(0) - chrono::Duration::minutes(1),
        at(3),
        &[ready],
    )
    .await;
    let ready_runtime = runtime_at(ready_root.path()).await.unwrap();
    assert_eq!(ready_runtime.occurrences[0].state, OccurrenceState::Pending);

    let missed_root = tempfile::tempdir().unwrap();
    let missed = once_definition(Uuid::new_v4(), at(0));
    scan_from(
        missed_root.path(),
        at(0) - chrono::Duration::minutes(1),
        at(20),
        &[missed],
    )
    .await;
    let missed_runtime = runtime_at(missed_root.path()).await.unwrap();
    assert!(missed_runtime.occurrences.iter().any(|item| {
        item.state == OccurrenceState::Terminal
            && item.result.as_ref().map(|result| &result.status)
                == Some(&OccurrenceResultStatus::Missed)
    }));
}

#[tokio::test]
async fn running_has_one_pending_and_later_deadlines_coalesce_into_it() {
    let root = tempfile::tempdir().unwrap();
    let automation = definition(Uuid::new_v4());
    let ids = scan_from(
        root.path(),
        at(0) - chrono::Duration::minutes(1),
        at(1),
        std::slice::from_ref(&automation),
    )
    .await;
    let occurrence_id = ids[0];
    mark_running_at(root.path(), occurrence_id, at(1))
        .await
        .unwrap();
    scan_if_active_at(root.path(), false, at(6), std::slice::from_ref(&automation))
        .await
        .unwrap();
    scan_if_active_at(
        root.path(),
        false,
        at(11),
        std::slice::from_ref(&automation),
    )
    .await
    .unwrap();
    let runtime = runtime_at(root.path()).await.unwrap();
    assert_eq!(runtime.occurrences.len(), 2);
    let pending = runtime
        .occurrences
        .iter()
        .find(|item| item.state == OccurrenceState::Pending)
        .unwrap();
    assert_eq!(pending.coalesced_count, Some(2));
    assert_eq!(pending.last_scheduled_for, Some(at(10)));
}

#[tokio::test]
async fn a_busy_session_keeps_pending_without_age_expiration() {
    let root = tempfile::tempdir().unwrap();
    let automation = definition(Uuid::new_v4());
    let occurrence_id = scan_from(
        root.path(),
        at(0) - chrono::Duration::minutes(1),
        at(1),
        std::slice::from_ref(&automation),
    )
    .await[0];
    scan_if_active_at(
        root.path(),
        false,
        at(30),
        std::slice::from_ref(&automation),
    )
    .await
    .unwrap();
    let runtime = runtime_at(root.path()).await.unwrap();
    let pending = runtime
        .occurrences
        .iter()
        .find(|item| item.id == occurrence_id)
        .unwrap();
    assert_eq!(pending.state, OccurrenceState::Pending);
    assert_eq!(pending.id, occurrence_id);
    assert_eq!(pending.coalesced_count, Some(7));
}

#[tokio::test]
async fn pause_crossing_a_deadline_keeps_the_checkpoint_for_resume() {
    let root = tempfile::tempdir().unwrap();
    let mut automation = definition(Uuid::new_v4());
    automation.schedule = AutomationSchedule::Once {
        local_datetime: at(0).naive_utc(),
        timezone: chrono_tz::UTC,
    };
    let before_deadline = at(0) - chrono::Duration::minutes(1);
    crate::services::automations::scan_and_advance_at(root.path(), before_deadline, &[])
        .await
        .unwrap();

    scan_if_active_at(root.path(), true, at(10), &[automation.clone()])
        .await
        .unwrap();
    assert_eq!(
        runtime_at(root.path()).await.unwrap().last_checked_at,
        before_deadline
    );

    scan_if_active_at(root.path(), false, at(10), &[automation])
        .await
        .unwrap();
    let runtime = runtime_at(root.path()).await.unwrap();
    assert_eq!(runtime.last_checked_at, at(10));
    assert!(runtime.occurrences.iter().any(|item| {
        item.state == OccurrenceState::Terminal
            && item
                .result
                .as_ref()
                .is_some_and(|result| result.status == OccurrenceResultStatus::Missed)
    }));
}

#[test]
fn pending_work_caps_the_scheduler_sleep_to_one_minute() {
    assert_eq!(
        sleep_until_next(&[], false, at(0), true),
        std::time::Duration::from_secs(60)
    );
}

#[test]
fn every_terminal_occurrence_is_selected_for_retry() {
    let mut terminal =
        crate::services::automations::AutomationOccurrence::pending(Uuid::new_v4(), at(0));
    terminal.state = OccurrenceState::Terminal;
    terminal.result = Some(result());
    let pending =
        crate::services::automations::AutomationOccurrence::pending(Uuid::new_v4(), at(1));
    let runtime = crate::services::automations::AutomationRuntime {
        schema_version: 1,
        last_checked_at: at(1),
        occurrences: vec![terminal.clone(), pending],
        retired_automation_ids: Vec::new(),
    };

    assert_eq!(terminal_ids(&runtime), vec![terminal.id]);
}

#[tokio::test]
async fn terminal_publication_is_replayed_once_and_completes_once() {
    let root = tempfile::tempdir().unwrap();
    let mut automation = definition(Uuid::new_v4());
    automation.schedule = AutomationSchedule::Once {
        local_datetime: at(0).naive_utc(),
        timezone: chrono_tz::UTC,
    };
    mutate_automations_at(root.path(), |items| {
        items.push(automation.clone());
        Ok(())
    })
    .await
    .unwrap();
    let occurrence_id = scan_from(
        root.path(),
        at(0) - chrono::Duration::minutes(1),
        at(1),
        std::slice::from_ref(&automation),
    )
    .await[0];
    mark_running_at(root.path(), occurrence_id, at(1))
        .await
        .unwrap();
    let result = result();
    mark_terminal_at(root.path(), occurrence_id, result.clone())
        .await
        .unwrap();

    append_history_at(
        &root.path().join("logs/wakeups.jsonl"),
        history_entry(occurrence_id, automation.id, &result),
    )
    .await
    .unwrap();
    publish_terminal_at(root.path(), occurrence_id)
        .await
        .unwrap();

    let history = all_history_at(&root.path().join("logs/wakeups.jsonl"), None)
        .await
        .unwrap();
    assert_eq!(history.len(), 1);
    let definitions = read_automations_at(root.path()).await.unwrap();
    assert_eq!(definitions[0].status, AutomationStatus::Completed);
    assert_eq!(definitions[0].revision, 2);
    assert!(runtime_at(root.path())
        .await
        .unwrap()
        .occurrences
        .is_empty());
}

#[tokio::test]
async fn deleted_definition_is_not_resurrected_when_running_finishes() {
    let root = tempfile::tempdir().unwrap();
    let automation = definition(Uuid::new_v4());
    mutate_automations_at(root.path(), |items| {
        items.push(automation.clone());
        Ok(())
    })
    .await
    .unwrap();
    let occurrence_id = scan_from(
        root.path(),
        at(0) - chrono::Duration::minutes(1),
        at(1),
        std::slice::from_ref(&automation),
    )
    .await[0];
    mark_running_at(root.path(), occurrence_id, at(1))
        .await
        .unwrap();
    mutate_automations_at(root.path(), |items| {
        items.clear();
        Ok(())
    })
    .await
    .unwrap();
    mark_terminal_at(root.path(), occurrence_id, result())
        .await
        .unwrap();
    publish_terminal_at(root.path(), occurrence_id)
        .await
        .unwrap();

    assert!(read_automations_at(root.path()).await.unwrap().is_empty());
    assert_eq!(
        all_history_at(&root.path().join("logs/wakeups.jsonl"), None)
            .await
            .unwrap()
            .len(),
        1
    );
}

#[tokio::test]
async fn startup_terminalizes_without_replaying_work() {
    let root = tempfile::tempdir().unwrap();
    let first = definition(Uuid::new_v4());
    let second = definition(Uuid::new_v4());
    let ids = scan_from(
        root.path(),
        at(0) - chrono::Duration::minutes(1),
        at(1),
        &[first, second],
    )
    .await;
    let occurrence_id = ids[0];
    mark_running_at(root.path(), occurrence_id, at(1))
        .await
        .unwrap();
    assert_eq!(ids.len(), 2);

    let recovered = recover_startup_at(root.path(), at(20)).await.unwrap();
    assert_eq!(recovered.len(), 2);
    let runtime = runtime_at(root.path()).await.unwrap();
    assert!(runtime.occurrences.iter().any(|item| {
        item.result.as_ref().is_some_and(|result| {
            result.status == OccurrenceResultStatus::Interrupted
                && result.error_code.as_deref() == Some("app_stopped")
        })
    }));
    assert!(runtime.occurrences.iter().any(|item| {
        item.result
            .as_ref()
            .is_some_and(|result| result.status == OccurrenceResultStatus::Missed)
    }));
}

#[tokio::test]
async fn interrupted_after_completion_reanchors_at_startup() {
    let root = tempfile::tempdir().unwrap();
    let mut automation = definition(Uuid::new_v4());
    automation.schedule = AutomationSchedule::AfterCompletion { delay_minutes: 10 };
    automation.anchor_at = Some(at(0));
    mutate_automations_at(root.path(), |items| {
        items.push(automation.clone());
        Ok(())
    })
    .await
    .unwrap();
    let occurrence_id = scan_from(
        root.path(),
        at(9),
        at(11),
        std::slice::from_ref(&automation),
    )
    .await[0];
    mark_running_at(root.path(), occurrence_id, at(11))
        .await
        .unwrap();

    let recovered = recover_startup_at(root.path(), at(20)).await.unwrap();
    publish_terminal_at(root.path(), recovered[0])
        .await
        .unwrap();

    let definitions = read_automations_at(root.path()).await.unwrap();
    assert_eq!(definitions[0].anchor_at, Some(at(20)));
    assert_eq!(definitions[0].revision, 2);
}

#[tokio::test]
async fn finalization_preserves_changes_made_while_running() {
    let root = tempfile::tempdir().unwrap();
    let automation = definition(Uuid::new_v4());
    mutate_automations_at(root.path(), |items| {
        items.push(automation.clone());
        Ok(())
    })
    .await
    .unwrap();
    let occurrence_id = scan_from(
        root.path(),
        at(0) - chrono::Duration::minutes(1),
        at(1),
        std::slice::from_ref(&automation),
    )
    .await[0];
    mark_running_at(root.path(), occurrence_id, at(1))
        .await
        .unwrap();
    mutate_automations_at(root.path(), |items| {
        items[0].name = "CI renommée".into();
        items[0].model = "gpt-6-astra".into();
        items[0].schedule = AutomationSchedule::Cron {
            expression: "*/20 * * * *".into(),
            timezone: chrono_tz::UTC,
        };
        items[0].revision += 1;
        Ok(())
    })
    .await
    .unwrap();
    mark_terminal_at(root.path(), occurrence_id, result())
        .await
        .unwrap();
    publish_terminal_at(root.path(), occurrence_id)
        .await
        .unwrap();

    let current = read_automations_at(root.path()).await.unwrap();
    assert_eq!(current[0].name, "CI renommée");
    assert_eq!(current[0].model, "gpt-6-astra");
    assert!(matches!(
        current[0].schedule,
        AutomationSchedule::Cron { ref expression, .. } if expression == "*/20 * * * *"
    ));
}

fn result() -> AutomationRunResult {
    AutomationRunResult {
        status: OccurrenceResultStatus::Ok,
        finished_at: at(4),
        error_code: None,
        session_id: Some("session-a".into()),
        tokens: Some(10),
        missed_count: None,
        first_scheduled_for: None,
        last_scheduled_for: None,
    }
}

fn history_entry(run_id: Uuid, automation_id: Uuid, result: &AutomationRunResult) -> HistoryEntry {
    HistoryEntry {
        run_id: Some(run_id),
        automation_id: automation_id.to_string(),
        scheduled_for: at(0).to_rfc3339(),
        finished_at: result.finished_at.to_rfc3339(),
        started_at: None,
        status: crate::models::WakeupRunStatus::Ok,
        error_code: None,
        session_id: result.session_id.clone(),
        tokens: result.tokens,
        missed_count: None,
        first_scheduled_for: None,
        last_scheduled_for: None,
    }
}
