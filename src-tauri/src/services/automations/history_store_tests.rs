use super::*;
use crate::models::WakeupRunStatus;
use chrono::{SecondsFormat, Utc};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use uuid::Uuid;

fn entry(id: Uuid, run_id: Option<Uuid>, minute: usize) -> HistoryEntry {
    let at = format!("2026-09-10T10:{:02}:00Z", minute % 60);
    HistoryEntry {
        run_id,
        automation_id: id.to_string(),
        scheduled_for: at.clone(),
        finished_at: at,
        started_at: None,
        status: WakeupRunStatus::Ok,
        error_code: None,
        session_id: Some("session-safe".into()),
        tokens: Some(12),
        missed_count: None,
        first_scheduled_for: None,
        last_scheduled_for: None,
    }
}

#[tokio::test]
async fn deduplicates_run_ids_and_reads_legacy_lines_without_sensitive_fields() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("wakeups.jsonl");
    let automation_id = Uuid::new_v4();
    let run_id = Uuid::new_v4();
    super::history_store::append_at(&path, entry(automation_id, Some(run_id), 1))
        .await
        .unwrap();
    super::history_store::append_at(&path, entry(automation_id, Some(run_id), 2))
        .await
        .unwrap();
    let legacy = format!(
        "{{\"wakeup_id\":\"{automation_id}\",\"scheduled_for\":\"2026-09-10T09:00:00Z\",\"fired_at\":\"2026-09-10T09:01:00Z\",\"status\":\"error\",\"error\":\"secret path\",\"provider_response\":\"secret\"}}\n"
    );
    let mut bytes = tokio::fs::read(&path).await.unwrap();
    bytes.extend_from_slice(legacy.as_bytes());
    tokio::fs::write(&path, bytes).await.unwrap();

    let page = super::history_store::page_at(&path, automation_id, None, None)
        .await
        .unwrap();
    assert_eq!(page.entries.len(), 2);
    assert!(page.entries.iter().any(|item| item.run_id.is_none()));
    let public = serde_json::to_string(&page.entries).unwrap();
    assert!(!public.contains("secret"));
    assert!(!public.contains("provider_response"));
}

#[tokio::test]
async fn pagination_is_bounded_and_an_expired_cursor_is_rejected() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("wakeups.jsonl");
    let automation_id = Uuid::new_v4();
    for minute in 0..25 {
        super::history_store::append_at(&path, entry(automation_id, Some(Uuid::new_v4()), minute))
            .await
            .unwrap();
    }
    let first = super::history_store::page_at(&path, automation_id, None, None)
        .await
        .unwrap();
    assert_eq!(first.entries.len(), 20);
    assert!(first.next_cursor.is_some());
    let capped = super::history_store::page_at(&path, automation_id, Some(500), None)
        .await
        .unwrap();
    assert_eq!(capped.entries.len(), 25);
    assert_eq!(
        super::history_store::page_at(&path, automation_id, None, Some("opaque-garbage"))
            .await
            .unwrap_err(),
        AutomationError::CursorExpired
    );
}

#[tokio::test]
async fn rotation_keeps_the_newest_half_and_expires_old_anchors() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("wakeups.jsonl");
    let automation_id = Uuid::new_v4();
    let first = entry(automation_id, Some(Uuid::new_v4()), 0);
    super::history_store::append_at(&path, first).await.unwrap();
    super::history_store::append_at(&path, entry(automation_id, Some(Uuid::new_v4()), 1))
        .await
        .unwrap();
    let cursor = super::history_store::page_at(&path, automation_id, Some(1), None)
        .await
        .unwrap()
        .next_cursor;
    for index in 2..=super::history_store::MAX_LINES {
        let mut item = entry(automation_id, Some(Uuid::new_v4()), index);
        item.finished_at = Utc::now()
            .checked_add_signed(chrono::Duration::seconds(index as i64))
            .unwrap()
            .to_rfc3339_opts(SecondsFormat::Secs, true);
        super::history_store::append_at(&path, item).await.unwrap();
    }
    let lines = String::from_utf8(tokio::fs::read(&path).await.unwrap())
        .unwrap()
        .lines()
        .count();
    assert_eq!(lines, super::history_store::ROTATED_LINES);
    assert_eq!(
        super::history_store::page_at(&path, automation_id, Some(1), cursor.as_deref())
            .await
            .unwrap_err(),
        AutomationError::CursorExpired
    );
}

#[tokio::test]
async fn appends_stay_incremental_until_rotation() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("incremental.jsonl");
    let automation_id = Uuid::new_v4();
    let reads = Arc::new(AtomicUsize::new(0));
    for minute in 0..super::history_store::MAX_LINES {
        let reads = Arc::clone(&reads);
        super::history_store_test_support::append_with_read_observer(
            &path,
            entry(automation_id, Some(Uuid::new_v4()), minute),
            move || {
                reads.fetch_add(1, Ordering::Relaxed);
            },
        )
        .await
        .unwrap();
    }
    assert_eq!(reads.load(Ordering::Relaxed), 1);
}

#[tokio::test]
async fn failed_rotation_keeps_the_previous_history() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("failure.jsonl");
    let automation_id = Uuid::new_v4();
    for minute in 0..super::history_store::MAX_LINES {
        super::history_store::append_at(&path, entry(automation_id, Some(Uuid::new_v4()), minute))
            .await
            .unwrap();
    }
    let before = tokio::fs::read(&path).await.unwrap();
    let result = super::history_store_test_support::append_with_atomic_writer(
        &path,
        entry(automation_id, Some(Uuid::new_v4()), 59),
        |_, _| async { Err("injected".into()) },
    )
    .await;

    assert!(result.is_err());
    assert_eq!(tokio::fs::read(&path).await.unwrap(), before);
}
