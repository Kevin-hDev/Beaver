use super::durable_fs::{retry_windows_sharing, OllamaFsError, OllamaFsErrorKind};
#[cfg(unix)]
use super::durable_fs::{OllamaDurableFs, PlatformOllamaDurableFs};
use super::durable_fs_test_support::{FailurePoint, ScriptedFs};
use super::journal::{OllamaJournalState, OllamaTransactionJournal};
use super::journal_store::OllamaJournalStore;
use crate::services::ollama_manager::fingerprint::{
    BundleFingerprint, OllamaVersion, Sha256Digest,
};
use crate::services::paths::ollama_paths;
use std::cell::Cell;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

fn journal() -> OllamaTransactionJournal {
    let digest = |value: &str| Sha256Digest::from_hex(&value.repeat(64)).unwrap();
    OllamaTransactionJournal::new(OllamaJournalState::Prepared {
        target: BundleFingerprint {
            version: OllamaVersion::parse("1.2.3").unwrap(),
            executable_sha256: digest("1"),
        },
        previous: BundleFingerprint {
            version: OllamaVersion::parse("1.2.2").unwrap(),
            executable_sha256: digest("2"),
        },
    })
}

fn store(fs: Arc<ScriptedFs>) -> OllamaJournalStore<ScriptedFs> {
    OllamaJournalStore::new(fs, ollama_paths(Path::new("/tmp/ollama-task-4")))
}

#[tokio::test]
async fn store_revalidates_after_the_final_parent_sync() {
    let fs = Arc::new(ScriptedFs::default());
    store(Arc::clone(&fs)).write_new(&journal()).await.unwrap();
    assert_eq!(
        fs.events(),
        [
            "read_tmp",
            "create_tmp",
            "write",
            "sync_file",
            "rename",
            "sync_parent",
            "read_final"
        ]
    );
}

#[tokio::test]
async fn every_durability_boundary_blocks_publication() {
    for point in [
        FailurePoint::CreateTmp,
        FailurePoint::Write,
        FailurePoint::SyncFile,
        FailurePoint::Rename,
        FailurePoint::SyncParent,
    ] {
        let fs = Arc::new(ScriptedFs::default());
        fs.fail_at(point);
        assert!(store(Arc::clone(&fs)).write_new(&journal()).await.is_err());
        assert!(!fs.events().contains(&"read_final"));
        assert!(fs.temp_is_absent());
    }
}

#[tokio::test]
async fn final_revalidation_rejects_a_tampered_document() {
    let fs = Arc::new(ScriptedFs::default());
    fs.set_final_override(b"not-json".to_vec());
    assert!(store(Arc::clone(&fs)).write_new(&journal()).await.is_err());
    assert!(fs.events().contains(&"read_final"));
}

#[tokio::test]
async fn replacement_calls_the_explicit_replace_primitive() {
    let fs = Arc::new(ScriptedFs::default());
    store(Arc::clone(&fs)).replace(&journal()).await.unwrap();
    assert!(fs.events().contains(&"replace"));
}

#[tokio::test]
async fn preexisting_tmp_is_rejected_without_writing() {
    let fs = Arc::new(ScriptedFs::default());
    fs.set_tmp(b"ambiguous".to_vec());
    assert!(store(Arc::clone(&fs)).write_new(&journal()).await.is_err());
    assert_eq!(fs.events(), ["read_tmp"]);
}

#[test]
fn sharing_retry_succeeds_after_two_violations() {
    let mut attempts = 0;
    let mut waits = Vec::new();
    let result = retry_windows_sharing(
        || {
            attempts += 1;
            if attempts < 3 {
                Err(OllamaFsError::new(OllamaFsErrorKind::SharingViolation))
            } else {
                Ok(())
            }
        },
        || false,
        |delay| waits.push(delay),
    );
    assert!(result.is_ok());
    assert_eq!(attempts, 3);
    assert_eq!(waits, [Duration::from_millis(50); 2]);
}

#[test]
fn permanent_sharing_retry_is_bounded_and_cancelable() {
    let mut waits = 0;
    let result = retry_windows_sharing(
        || Err::<(), _>(OllamaFsError::new(OllamaFsErrorKind::SharingViolation)),
        || false,
        |_| waits += 1,
    );
    assert!(result.is_err());
    assert_eq!(waits, 40);

    let waits = Cell::new(0);
    let result = retry_windows_sharing(
        || Err::<(), _>(OllamaFsError::new(OllamaFsErrorKind::SharingViolation)),
        || waits.get() >= 2,
        |_| waits.set(waits.get() + 1),
    );
    assert!(result.unwrap_err().is_cancelled());
    assert_eq!(waits.get(), 2);
}

#[test]
fn non_sharing_errors_are_not_retried() {
    let mut waits = 0;
    let result = retry_windows_sharing(
        || Err::<(), _>(OllamaFsError::new(OllamaFsErrorKind::Other)),
        || false,
        |_| waits += 1,
    );
    assert!(result.is_err());
    assert_eq!(waits, 0);
}

#[cfg(unix)]
#[test]
fn unix_atomic_primitive_distinguishes_new_and_replace() {
    let root = tempfile::tempdir().unwrap();
    let fs = PlatformOllamaDurableFs;
    let tmp = root.path().join("journal.tmp");
    let final_path = root.path().join("journal.json");

    fs.write_new_atomic(&tmp, &final_path, b"first").unwrap();
    assert_eq!(std::fs::read(&final_path).unwrap(), b"first");
    let error = fs
        .write_new_atomic(&tmp, &final_path, b"second")
        .unwrap_err();
    assert_eq!(error.kind(), OllamaFsErrorKind::AlreadyExists);
    assert_eq!(std::fs::read(&final_path).unwrap(), b"first");

    fs.replace_atomic(&tmp, &final_path, b"second").unwrap();
    assert_eq!(std::fs::read(&final_path).unwrap(), b"second");
    fs.remove_file_durable(&final_path).unwrap();
    assert!(!final_path.exists());
}
