use super::*;

fn line(id: &str, fired_at: &str, status: WakeupRunStatus) -> String {
    serde_json::to_string(&WakeupRun {
        wakeup_id: id.into(),
        scheduled_for: "2026-05-17T08:00:00+02:00".into(),
        fired_at: fired_at.into(),
        status,
        error_code: None,
        error: None,
        session_id: None,
        tokens: None,
    })
    .unwrap()
}

#[test]
fn parse_runs_filters_and_sorts_newest_first() {
    let content = format!(
        "{}\n{}\n",
        line("a", "2026-05-17T08:00:00Z", WakeupRunStatus::Ok),
        line("b", "2026-05-17T09:00:00Z", WakeupRunStatus::Missed)
    );
    let runs = parse_runs(&content, None);
    assert_eq!(runs[0].wakeup_id, "b");
    assert_eq!(parse_runs(&content, Some("a")).len(), 1);
}

#[test]
fn generic_error_does_not_return_raw_message() {
    assert_eq!(
        generic_error_code("token secret leaked"),
        WakeupRunErrorCode::Failed
    );
    assert_eq!(
        generic_error_code("Ollama HTTP 500"),
        WakeupRunErrorCode::OllamaUnavailable
    );
}

#[test]
fn generic_error_maps_known_failures() {
    assert_eq!(
        generic_error_code("RATE LIMIT hit"),
        WakeupRunErrorCode::RateLimited
    );
    assert_eq!(
        generic_error_code("401 unauthorized"),
        WakeupRunErrorCode::AuthenticationFailed
    );
    assert_eq!(
        generic_error_code("invalid api key sk-xxx"),
        WakeupRunErrorCode::Failed
    );
}

#[test]
fn generic_error_never_returns_sensitive_input() {
    let sensitive_inputs = [
        "Bearer sk-abcd1234efgh",
        "Connection refused to 127.0.0.1:11434",
        "panic at src/main.rs:42",
        "/Users/secret/.config/keys.json not found",
    ];
    for input in sensitive_inputs {
        assert_eq!(generic_error_code(input), WakeupRunErrorCode::Failed);
    }
}

#[test]
fn admission_refusals_have_stable_error_codes() {
    use crate::services::work_registry::ServiceWorkAdmissionError;

    assert_eq!(
        refusal_error_code(ServiceWorkAdmissionError::Closing),
        WakeupRunErrorCode::SchedulerStopping
    );
    assert_eq!(
        refusal_error_code(ServiceWorkAdmissionError::Capacity),
        WakeupRunErrorCode::CapacityReached
    );
}

#[test]
fn missed_occurrence_has_a_typed_missed_outcome() {
    let entry = missed_entry("daily", chrono::Local::now());

    assert_eq!(entry.status, WakeupRunStatus::Missed);
    assert_eq!(
        entry.error_code,
        Some(WakeupRunErrorCode::MissedUnavailable)
    );
    assert!(entry.error.is_none());
}

#[test]
fn legacy_error_is_read_but_never_serialized_back_to_the_frontend() {
    let raw = r#"{"wakeup_id":"daily","scheduled_for":"2026-08-12T10:00:00+02:00","fired_at":"2026-08-12T08:00:00Z","status":"error","error":"/private/config.json","session_id":null,"tokens":null}"#;
    let entry: WakeupRun = serde_json::from_str(raw).unwrap();

    assert_eq!(entry.error.as_deref(), Some("/private/config.json"));
    assert!(!serde_json::to_string(&entry).unwrap().contains("/private"));
}

#[tokio::test]
async fn repeated_occurrence_is_written_only_once() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("wakeups.jsonl");
    let entry = error_entry(
        "daily",
        chrono::Local::now(),
        WakeupRunErrorCode::CapacityReached,
    );

    append_at(&path, entry.clone()).await.unwrap();
    append_at(&path, entry).await.unwrap();

    assert_eq!(list_runs_at(&path, None).await.unwrap().len(), 1);
}

#[tokio::test]
async fn concurrent_reader_never_observes_partial_rotation() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("wakeups.jsonl");
    tokio::fs::write(&path, seeded_log()).await.unwrap();
    let (rotation_started_tx, rotation_started_rx) = tokio::sync::oneshot::channel();
    let (resume_rotation_tx, resume_rotation_rx) = tokio::sync::oneshot::channel();
    let writer_path = path.clone();
    let writer = tokio::spawn(async move {
        append_at_with_atomic_writer(&writer_path, new_entry(), move |path, bytes| async move {
            let _ = rotation_started_tx.send(());
            let _ = resume_rotation_rx.await;
            crate::services::private_store::atomic_write_async(path, bytes).await
        })
        .await
    });
    rotation_started_rx.await.unwrap();

    let reader_path = path.clone();
    let reader = tokio::spawn(async move { list_runs_at(&reader_path, None).await });
    tokio::task::yield_now().await;
    assert!(!reader.is_finished(), "reader escaped the journal lock");

    resume_rotation_tx.send(()).unwrap();
    writer.await.unwrap().unwrap();
    let runs = reader.await.unwrap().unwrap();
    assert_eq!(runs.len(), MAX_LINES);
    assert_eq!(runs[0].wakeup_id, "new-entry");
}

#[tokio::test]
async fn failed_rotation_keeps_the_previous_journal_unchanged() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("wakeups.jsonl");
    let original = seeded_log();
    tokio::fs::write(&path, &original).await.unwrap();

    let result = append_at_with_atomic_writer(&path, new_entry(), |_, _| async {
        Err("injected-rotation-failure".to_string())
    })
    .await;

    assert_eq!(result.unwrap_err(), "injected-rotation-failure");
    assert_eq!(tokio::fs::read(&path).await.unwrap(), original);
}

fn seeded_log() -> Vec<u8> {
    (0..MAX_LINES)
        .map(|index| {
            serde_json::to_string(&WakeupRun {
                wakeup_id: format!("w{index}"),
                scheduled_for: format!("2026-08-12T10:{:02}:00+02:00", index % 60),
                fired_at: format!("2026-08-12T08:{:02}:00Z", index % 60),
                status: WakeupRunStatus::Error,
                error_code: Some(WakeupRunErrorCode::Failed),
                error: None,
                session_id: None,
                tokens: None,
            })
            .unwrap()
                + "\n"
        })
        .collect::<String>()
        .into_bytes()
}

fn new_entry() -> WakeupRun {
    WakeupRun {
        wakeup_id: "new-entry".into(),
        scheduled_for: "2026-08-12T12:00:00+02:00".into(),
        fired_at: "2026-08-12T10:00:00Z".into(),
        status: WakeupRunStatus::Error,
        error_code: Some(WakeupRunErrorCode::Failed),
        error: None,
        session_id: None,
        tokens: None,
    }
}

#[test]
fn parse_runs_is_bounded_before_collection() {
    let mut content = String::new();
    for i in 0..600 {
        let fired_at = format!("2026-05-17T{i:04}:00:00Z");
        content.push_str(&line(&format!("w{i}"), &fired_at, WakeupRunStatus::Ok));
        content.push('\n');
    }
    let runs = parse_runs(&content, None);
    assert_eq!(runs.len(), MAX_LINES);
    assert!(runs.iter().all(|run| run.wakeup_id != "w0"));
}

#[test]
fn ids_are_ascii_bounded_and_control_characters_are_removed() {
    let id = format!("{}\nsecret", "a".repeat(MAX_ID_CHARS + 20));
    let safe = safe_id(&id);
    assert_eq!(safe.len(), MAX_ID_CHARS);
    assert!(!safe.contains('\n'));
}

#[test]
fn oversized_lines_are_never_deserialized() {
    let oversized = format!("{{\"wakeup_id\":\"{}\"}}", "a".repeat(MAX_LOG_LINE_BYTES));
    assert!(parse_runs(&oversized, None).is_empty());
}
