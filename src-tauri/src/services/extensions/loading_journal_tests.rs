use super::loading_marker::{self, JournalRead, MarkerRead};
use std::collections::HashSet;
use std::sync::{mpsc, Arc};
use std::time::Duration;

const HOST_ID: &str = "com.example.host";
const UI_ID: &str = "com.example.ui";

#[test]
fn v1_marker_migrates_in_memory_to_the_v2_host_entry() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("extension-loading.json");
    std::fs::write(
        &path,
        br#"{"version":1,"extensionId":"com.example.host","stage":"activate","startedAt":"2026-09-03T00:00:00Z","attempts":2}"#,
    )
    .unwrap();

    let JournalRead::Valid(journal) = loading_marker::read_journal_at(&path) else {
        panic!("valid migrated journal expected");
    };
    assert_eq!(journal.version(), 2);
    assert_eq!(journal.host().unwrap().extension_id, HOST_ID);
    assert_eq!(journal.host().unwrap().stage, "activate");
    assert!(journal.ui().is_none());
    assert!(matches!(
        loading_marker::read_at(&path),
        MarkerRead::Valid(_)
    ));
    assert!(std::fs::read_to_string(path)
        .unwrap()
        .contains("\"version\":1"));
}

#[test]
fn host_and_ui_updates_preserve_each_other_and_remove_only_when_both_empty() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("extension-loading.json");

    loading_marker::start_at(&path, HOST_ID, 1).unwrap();
    loading_marker::ui_start_at(&path, UI_ID, 1).unwrap();
    loading_marker::advance_at(&path, HOST_ID, "register").unwrap();
    loading_marker::ui_advance_at(&path, UI_ID, "activate").unwrap();

    let JournalRead::Valid(journal) = loading_marker::read_journal_at(&path) else {
        panic!("journal expected");
    };
    assert_eq!(journal.host().unwrap().stage, "register");
    assert_eq!(journal.ui().unwrap().stage, "activate");

    loading_marker::ui_complete_at(&path, UI_ID).unwrap();
    assert!(path.exists());
    assert!(loading_marker::read_journal_at(&path)
        .valid()
        .unwrap()
        .ui()
        .is_none());
    loading_marker::discard_at(&path).unwrap();
    assert!(!path.exists());
}

#[test]
fn concurrent_host_and_ui_transactions_cannot_publish_stale_copies() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("extension-loading.json");
    let (entered_tx, entered_rx) = mpsc::sync_channel(0);
    let (release_tx, release_rx) = mpsc::sync_channel(0);
    let host_path = path.clone();
    let host = std::thread::spawn(move || {
        super::loading_journal_store::transaction(&host_path, false, |read| {
            let JournalRead::Missing = read else {
                panic!("empty journal expected");
            };
            entered_tx.send(()).unwrap();
            release_rx.recv().unwrap();
            let mut journal = super::loading_journal_format::LoadingJournal::empty();
            *journal.host_mut() =
                Some(super::loading_marker_format::LoadingMarker::new_host(HOST_ID, 1).unwrap());
            Ok(Some(journal))
        })
    });
    entered_rx.recv().unwrap();

    let (ui_ready_tx, ui_ready_rx) = mpsc::sync_channel(0);
    let (ui_done_tx, ui_done_rx) = mpsc::sync_channel(0);
    let ui_path = path.clone();
    let ui = std::thread::spawn(move || {
        ui_ready_tx.send(()).unwrap();
        let result = loading_marker::ui_start_at(&ui_path, UI_ID, 1);
        ui_done_tx.send(()).unwrap();
        result
    });
    ui_ready_rx.recv().unwrap();
    assert!(matches!(
        ui_done_rx.recv_timeout(Duration::from_millis(100)),
        Err(mpsc::RecvTimeoutError::Timeout)
    ));
    release_tx.send(()).unwrap();
    host.join().unwrap().unwrap();
    ui_done_rx.recv().unwrap();
    ui.join().unwrap().unwrap();

    let journal = loading_marker::read_journal_at(&path).valid().unwrap();
    assert_eq!(journal.host().unwrap().extension_id, HOST_ID);
    assert_eq!(journal.ui().unwrap().extension_id, UI_ID);
}

#[test]
fn completion_and_neighbor_ui_mutation_share_the_same_transaction() {
    let directory = tempfile::tempdir().unwrap();
    let path = Arc::new(directory.path().join("extension-loading.json"));
    loading_marker::start_at(path.as_ref(), HOST_ID, 1).unwrap();
    loading_marker::ui_start_at(path.as_ref(), UI_ID, 1).unwrap();
    let preserved = loading_marker::preserve_at(path.as_ref());
    let (entered_tx, entered_rx) = mpsc::sync_channel(0);
    let (release_tx, release_rx) = mpsc::sync_channel(0);
    let ui_path = Arc::clone(&path);
    let ui = std::thread::spawn(move || {
        super::loading_journal_store::transaction(ui_path.as_ref(), false, |read| {
            let JournalRead::Valid(mut journal) = read else {
                panic!("journal expected");
            };
            entered_tx.send(()).unwrap();
            release_rx.recv().unwrap();
            journal.ui_mut().as_mut().unwrap().stage = "mount".to_string();
            Ok(Some(journal))
        })
    });
    entered_rx.recv().unwrap();

    let (complete_ready_tx, complete_ready_rx) = mpsc::sync_channel(0);
    let (complete_done_tx, complete_done_rx) = mpsc::sync_channel(0);
    let complete_path = Arc::clone(&path);
    let completion = std::thread::spawn(move || {
        complete_ready_tx.send(()).unwrap();
        let result = loading_marker::complete_at(
            complete_path.as_ref(),
            preserved,
            &HashSet::from([HOST_ID.to_string()]),
            None,
        );
        complete_done_tx.send(()).unwrap();
        result
    });
    complete_ready_rx.recv().unwrap();
    assert!(matches!(
        complete_done_rx.recv_timeout(Duration::from_millis(100)),
        Err(mpsc::RecvTimeoutError::Timeout)
    ));
    release_tx.send(()).unwrap();
    ui.join().unwrap().unwrap();
    complete_done_rx.recv().unwrap();
    completion.join().unwrap().unwrap();

    let journal = loading_marker::read_journal_at(path.as_ref())
        .valid()
        .unwrap();
    assert!(journal.host().is_none());
    assert_eq!(journal.ui().unwrap().stage, "mount");
}

#[test]
fn clear_and_neighbor_host_mutation_share_the_same_transaction() {
    let directory = tempfile::tempdir().unwrap();
    let path = Arc::new(directory.path().join("extension-loading.json"));
    loading_marker::start_at(path.as_ref(), HOST_ID, 1).unwrap();
    loading_marker::ui_start_at(path.as_ref(), UI_ID, 1).unwrap();
    let (entered_tx, entered_rx) = mpsc::sync_channel(0);
    let (release_tx, release_rx) = mpsc::sync_channel(0);
    let host_path = Arc::clone(&path);
    let host = std::thread::spawn(move || {
        super::loading_journal_store::transaction(host_path.as_ref(), false, |read| {
            let JournalRead::Valid(mut journal) = read else {
                panic!("journal expected");
            };
            entered_tx.send(()).unwrap();
            release_rx.recv().unwrap();
            journal.host_mut().as_mut().unwrap().stage = "register".to_string();
            Ok(Some(journal))
        })
    });
    entered_rx.recv().unwrap();

    let (clear_ready_tx, clear_ready_rx) = mpsc::sync_channel(0);
    let (clear_done_tx, clear_done_rx) = mpsc::sync_channel(0);
    let clear_path = Arc::clone(&path);
    let clear = std::thread::spawn(move || {
        clear_ready_tx.send(()).unwrap();
        let result = loading_marker::ui_complete_at(clear_path.as_ref(), UI_ID);
        clear_done_tx.send(()).unwrap();
        result
    });
    clear_ready_rx.recv().unwrap();
    assert!(matches!(
        clear_done_rx.recv_timeout(Duration::from_millis(100)),
        Err(mpsc::RecvTimeoutError::Timeout)
    ));
    release_tx.send(()).unwrap();
    host.join().unwrap().unwrap();
    clear_done_rx.recv().unwrap();
    clear.join().unwrap().unwrap();

    let journal = loading_marker::read_journal_at(path.as_ref())
        .valid()
        .unwrap();
    assert_eq!(journal.host().unwrap().stage, "register");
    assert!(journal.ui().is_none());
}

#[test]
fn host_completion_restores_semantics_without_replaying_bytes_or_losing_new_ui_state() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("extension-loading.json");
    std::fs::write(
        &path,
        br#"{ "version": 1, "extensionId": "com.example.host", "stage": "activate", "startedAt": "2026-09-03T00:00:00Z", "attempts": 1 }"#,
    )
    .unwrap();
    let preserved = loading_marker::preserve_at(&path);

    loading_marker::ui_start_at(&path, UI_ID, 1).unwrap();
    loading_marker::ui_advance_at(&path, UI_ID, "mount").unwrap();
    loading_marker::start_at(&path, "com.example.neighbor", 1).unwrap();
    loading_marker::complete_at(
        &path,
        preserved,
        &HashSet::from(["com.example.neighbor".to_string()]),
        None,
    )
    .unwrap();

    let bytes = std::fs::read_to_string(&path).unwrap();
    assert!(!bytes.contains("{ \"version\": 1"));
    let journal = loading_marker::read_journal_at(&path).valid().unwrap();
    assert_eq!(journal.host().unwrap().extension_id, HOST_ID);
    assert_eq!(journal.host().unwrap().stage, "activate");
    assert_eq!(journal.ui().unwrap().stage, "mount");
}

#[test]
fn journal_is_atomic_bounded_and_worst_valid_shape_fits() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("extension-loading.json");
    let identifier = format!("a.{}", "b".repeat(94));
    loading_marker::start_at(&path, &identifier, 3).unwrap();
    loading_marker::ui_start_at(&path, &identifier, 3).unwrap();
    assert!(std::fs::metadata(&path).unwrap().len() <= loading_marker::MAX_MARKER_BYTES as u64);

    loading_marker::ui_advance_fail_before_replace_at(&path, &identifier, "mount").unwrap_err();
    assert_eq!(
        loading_marker::read_journal_at(&path)
            .valid()
            .unwrap()
            .ui()
            .unwrap()
            .stage,
        "contract"
    );

    std::fs::write(&path, vec![b'x'; loading_marker::MAX_MARKER_BYTES + 1]).unwrap();
    assert!(matches!(
        loading_marker::read_journal_at(&path),
        JournalRead::Invalid
    ));
}

#[test]
fn security_journal_rejects_unknown_duplicate_and_invalid_fields() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("extension-loading.json");
    let invalid = [
        r#"{"version":2,"extra":true}"#,
        r#"{"version":2,"version":2}"#,
        r#"{"version":2,"host":{"extensionId":"com.example.host","stage":"import","startedAt":"2026-09-03T00:00:00Z","attempts":1,"extra":true}}"#,
        r#"{"version":2,"ui":{"extensionId":"../escape","stage":"activate","startedAt":"2026-09-03T00:00:00Z","attempts":1}}"#,
        r#"{"version":2,"ui":{"extensionId":"com.example.ui","stage":"register","startedAt":"2026-09-03T00:00:00Z","attempts":1}}"#,
        r#"{"version":2,"host":{"extensionId":"com.example.host","stage":"import","startedAt":"not-a-date","attempts":1}}"#,
        r#"{"version":2,"host":{"extensionId":"com.example.host","stage":"import","startedAt":"2026-09-03T00:00:00Z","attempts":0}}"#,
    ];
    for bytes in invalid {
        std::fs::write(&path, bytes).unwrap();
        assert!(matches!(
            loading_marker::read_journal_at(&path),
            JournalRead::Invalid
        ));
    }
}

trait JournalReadTestExt {
    fn valid(self) -> Option<super::loading_journal_format::LoadingJournal>;
}

impl JournalReadTestExt for JournalRead {
    fn valid(self) -> Option<super::loading_journal_format::LoadingJournal> {
        match self {
            JournalRead::Valid(journal) => Some(journal),
            JournalRead::Missing | JournalRead::Invalid => None,
        }
    }
}
