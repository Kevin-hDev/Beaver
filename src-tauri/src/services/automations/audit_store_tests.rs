use super::*;
use crate::models::{AutomationSchedule, AutomationStatus, AutomationTarget, WakeupRunStatus};
use chrono::{TimeZone, Utc};
use serde_json::Value;
use uuid::Uuid;

fn actor() -> AutomationActor {
    AutomationActor {
        origin: AutomationOrigin::Session,
        session_or_channel_id: "audit-session".into(),
        current_automation_id: None,
    }
}

fn input(secret: &str) -> CreateAutomation {
    CreateAutomation {
        name: "CI".into(),
        description: None,
        prompt: secret.into(),
        target: AutomationTarget::ResumeSession {
            session_id: "audit-session".into(),
        },
        provider: "codex-oauth".into(),
        model: "gpt-5.6-luna".into(),
        schedule: AutomationSchedule::AfterCompletion { delay_minutes: 5 },
        status: AutomationStatus::Active,
    }
}

#[tokio::test]
async fn management_actions_are_audited_without_prompts_or_results() {
    let root = tempfile::tempdir().unwrap();
    let now = Utc.with_ymd_and_hms(2026, 9, 10, 10, 0, 0).unwrap();
    let created = create_at(root.path(), &actor(), input("PROMPT_SECRET"), now)
        .await
        .unwrap();
    let id = created.definition.id;
    list_at(root.path(), &actor(), now).await.unwrap();
    get_at(root.path(), &actor(), id, now).await.unwrap();
    update_at(
        root.path(),
        &actor(),
        id,
        UpdateAutomation {
            prompt: Some("UPDATED_PROMPT_SECRET".into()),
            ..Default::default()
        },
        now,
    )
    .await
    .unwrap();
    super::history_store::append_at(
        &root.path().join("logs/wakeups.jsonl"),
        HistoryEntry {
            run_id: Some(Uuid::new_v4()),
            automation_id: id.to_string(),
            scheduled_for: now.to_rfc3339(),
            finished_at: now.to_rfc3339(),
            started_at: None,
            status: WakeupRunStatus::Ok,
            error_code: None,
            session_id: None,
            tokens: None,
            missed_count: None,
            first_scheduled_for: None,
            last_scheduled_for: None,
        },
    )
    .await
    .unwrap();
    delete_at(root.path(), &actor(), id).await.unwrap();
    let history = super::service::history_at(
        root.path(),
        &actor(),
        HistoryQuery {
            automation_id: id,
            limit: None,
            cursor: None,
        },
    )
    .await
    .unwrap();
    assert_eq!(history.entries.len(), 1);
    assert_eq!(
        get_at(root.path(), &actor(), id, now).await.unwrap_err(),
        AutomationError::NotFound
    );

    let raw = tokio::fs::read_to_string(root.path().join("logs/automation-audit.jsonl"))
        .await
        .unwrap();
    assert!(!raw.contains("PROMPT_SECRET"));
    let entries = raw
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).unwrap())
        .collect::<Vec<_>>();
    for action in ["create", "list", "get", "update", "history", "delete"] {
        assert!(entries.iter().any(|entry| entry["action"] == action));
    }
    let update = entries
        .iter()
        .find(|entry| entry["action"] == "update" && entry["result"] == "intent")
        .unwrap();
    assert_eq!(update["fields"], serde_json::json!(["prompt"]));
    assert!(!raw.contains("UPDATED_PROMPT_SECRET"));
    assert!(entries
        .iter()
        .any(|entry| entry["action"] == "get" && entry["result"] == "not_found"));
    assert!(entries.iter().any(|entry| {
        entry["action"] == "list" && entry["result"] == "ok" && entry["count"] == 1
    }));
    assert!(entries.iter().all(|entry| {
        Uuid::parse_str(entry["operation_id"].as_str().unwrap())
            .is_ok_and(|id| id.get_version_num() == 4)
    }));
}

#[tokio::test]
async fn an_unwritable_audit_fails_closed_before_read_or_mutation() {
    let root = tempfile::tempdir().unwrap();
    tokio::fs::write(root.path().join("logs"), b"not-a-directory")
        .await
        .unwrap();
    assert_eq!(
        create_at(root.path(), &actor(), input("safe"), Utc::now())
            .await
            .unwrap_err(),
        AutomationError::AuditUnavailable
    );
    assert!(!root.path().join("automations.json").exists());
}

#[tokio::test]
async fn audit_rotation_keeps_only_the_newest_half() {
    let root = tempfile::tempdir().unwrap();
    let first = super::audit_store::begin(root.path(), &actor(), "list", None, Vec::new())
        .await
        .unwrap();
    for _ in 1..=super::audit_store::MAX_LINES {
        super::audit_store::begin(root.path(), &actor(), "list", None, Vec::new())
            .await
            .unwrap();
    }
    let raw = tokio::fs::read_to_string(root.path().join("logs/automation-audit.jsonl"))
        .await
        .unwrap();
    assert_eq!(raw.lines().count(), super::audit_store::ROTATED_LINES);
    assert!(!raw.contains(&first.to_string()));
}
