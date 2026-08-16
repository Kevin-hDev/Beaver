use super::durable_fs::OllamaDurableFs;
#[cfg(unix)]
use super::durable_fs::PlatformOllamaDurableFs;
use super::durable_fs::{
    retry_windows_sharing, sync_parent_pair, validate_wide_units, windows_file_flush_access,
    OllamaFsError, OllamaFsErrorKind, OllamaFsOperation, WINDOWS_PARENT_FLUSH_ACCESS,
};
use super::durable_fs_test_support::{ExpectedCall, FailurePoint, ScriptedFs};
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

fn scripted(calls: impl IntoIterator<Item = ExpectedCall>) -> Arc<ScriptedFs> {
    Arc::new(ScriptedFs::scripted_at(
        Path::new("/tmp/ollama-task-4"),
        calls,
    ))
}

#[tokio::test]
async fn store_revalidates_after_the_final_parent_sync() {
    let fs = scripted([
        ExpectedCall::CreateDirectory,
        ExpectedCall::ReadTmp,
        ExpectedCall::WriteNew,
        ExpectedCall::ReadFinal,
    ]);
    store(Arc::clone(&fs)).write_new(&journal()).await.unwrap();
    assert_eq!(
        fs.events(),
        [
            "create_directory",
            "read_tmp",
            "write_new",
            "create_tmp",
            "write",
            "sync_file",
            "rename",
            "sync_parent",
            "read_final"
        ]
    );
    fs.finish();
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
        let fs = scripted([
            ExpectedCall::CreateDirectory,
            ExpectedCall::ReadTmp,
            ExpectedCall::WriteNew,
        ]);
        fs.fail_at(point);
        assert!(store(Arc::clone(&fs)).write_new(&journal()).await.is_err());
        assert!(!fs.events().contains(&"read_final"));
        if point == FailurePoint::CreateTmp {
            assert!(fs.temp_is_absent());
        } else if point == FailurePoint::SyncParent {
            assert!(fs.temp_is_absent());
            assert!(fs.final_is_present());
        } else {
            assert!(!fs.temp_is_absent());
            assert!(!fs.final_is_present());
        }
        fs.finish();
    }
}

#[tokio::test]
async fn sync_parent_failure_happens_after_fake_publication() {
    let fs = scripted([
        ExpectedCall::CreateDirectory,
        ExpectedCall::ReadTmp,
        ExpectedCall::WriteNew,
    ]);
    fs.fail_at(FailurePoint::SyncParent);

    assert!(store(Arc::clone(&fs)).write_new(&journal()).await.is_err());
    assert!(fs.temp_is_absent());
    assert!(fs.final_is_present());
    fs.finish();
}

#[tokio::test]
async fn final_revalidation_rejects_a_tampered_document() {
    let fs = scripted([
        ExpectedCall::CreateDirectory,
        ExpectedCall::ReadTmp,
        ExpectedCall::WriteNew,
        ExpectedCall::ReadFinal,
    ]);
    fs.set_final_override(b"not-json".to_vec());
    assert!(store(Arc::clone(&fs)).write_new(&journal()).await.is_err());
    assert!(fs.events().contains(&"read_final"));
    fs.finish();
}

#[tokio::test]
async fn replacement_calls_the_explicit_replace_primitive() {
    let fs = scripted([
        ExpectedCall::CreateDirectory,
        ExpectedCall::ReadTmp,
        ExpectedCall::Replace,
        ExpectedCall::ReadFinal,
    ]);
    store(Arc::clone(&fs)).replace(&journal()).await.unwrap();
    assert!(fs.events().contains(&"replace"));
    fs.finish();
}

#[tokio::test]
async fn replacement_does_not_delegate_to_the_new_publication() {
    let fs = scripted([
        ExpectedCall::CreateDirectory,
        ExpectedCall::ReadTmp,
        ExpectedCall::Replace,
        ExpectedCall::ReadFinal,
    ]);
    store(Arc::clone(&fs)).replace(&journal()).await.unwrap();
    assert!(!fs.events().contains(&"write_new"));
    fs.finish();
}

#[tokio::test]
async fn preexisting_tmp_is_rejected_without_writing() {
    let fs = scripted([ExpectedCall::CreateDirectory, ExpectedCall::ReadTmp]);
    fs.set_tmp(b"ambiguous".to_vec());
    assert!(store(Arc::clone(&fs)).write_new(&journal()).await.is_err());
    assert_eq!(fs.events(), ["create_directory", "read_tmp"]);
    fs.finish();
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

#[test]
fn standard_io_errors_preserve_kind_and_raw_os_code() {
    #[cfg(windows)]
    let code = 5;
    #[cfg(unix)]
    let code = libc::EACCES;
    let error = std::io::Error::from_raw_os_error(code);
    let evidence = OllamaFsError::from_io(&error);

    assert_eq!(evidence.kind(), OllamaFsErrorKind::PermissionDenied);
    assert_eq!(evidence.os_code(), Some(code as u32));
}

#[test]
#[should_panic(expected = "unexpected fake FS call")]
fn scripted_fake_rejects_an_unexpected_call() {
    let fs = ScriptedFs::default();
    let _ = fs.read_bounded(Path::new("journal.json"), 64);
}

#[cfg(unix)]
#[test]
fn unix_publication_rejects_a_destination_created_after_preflight() {
    let root = tempfile::tempdir().unwrap();
    let fs = PlatformOllamaDurableFs;
    let tmp = root.path().join("journal.tmp");
    let final_path = root.path().join("journal.json");
    fs.write_new_atomic_with_hook(&tmp, &final_path, b"candidate", || {
        std::fs::write(&final_path, b"concurrent").unwrap();
    })
    .unwrap_err();
    assert_eq!(std::fs::read(&final_path).unwrap(), b"concurrent");
}

#[test]
fn publication_syncs_source_then_destination_parent_once() {
    let root = tempfile::tempdir().unwrap();
    let source_dir = root.path().join("source");
    let destination_dir = root.path().join("destination");
    std::fs::create_dir_all(&source_dir).unwrap();
    std::fs::create_dir_all(&destination_dir).unwrap();
    let source = source_dir.join("journal.tmp");
    let destination = destination_dir.join("journal.json");
    let mut seen = Vec::new();
    sync_parent_pair(&source, &destination, |parent| {
        seen.push(parent.to_path_buf());
        Ok::<_, OllamaFsError>(())
    })
    .unwrap();
    assert_eq!(seen, vec![source_dir.clone(), destination_dir]);

    let mut same_parent = Vec::new();
    sync_parent_pair(&source, &source_dir.join("other.json"), |parent| {
        same_parent.push(parent.to_path_buf());
        Ok::<_, OllamaFsError>(())
    })
    .unwrap();
    assert_eq!(same_parent, vec![source_dir]);
}

#[cfg(unix)]
#[test]
fn unix_publication_supports_distinct_source_and_destination_parents() {
    let root = tempfile::tempdir().unwrap();
    let source_dir = root.path().join("source");
    let destination_dir = root.path().join("destination");
    std::fs::create_dir_all(&source_dir).unwrap();
    std::fs::create_dir_all(&destination_dir).unwrap();
    let source = source_dir.join("journal.tmp");
    let destination = destination_dir.join("journal.json");

    PlatformOllamaDurableFs
        .write_new_atomic(&source, &destination, b"cross-directory")
        .unwrap();
    assert_eq!(std::fs::read(destination).unwrap(), b"cross-directory");
    assert!(!source.exists());
}

#[test]
fn windows_parent_flush_uses_generic_write() {
    assert_eq!(WINDOWS_PARENT_FLUSH_ACCESS, 0x4000_0000);
}

#[test]
fn windows_file_flush_uses_the_same_write_access_contract() {
    assert_eq!(windows_file_flush_access(), 0x4000_0000);
}

#[test]
fn native_error_evidence_preserves_the_raw_os_code() {
    let error = OllamaFsError::from_os_code(OllamaFsErrorKind::PermissionDenied, 5);

    assert_eq!(error.kind(), OllamaFsErrorKind::PermissionDenied);
    assert_eq!(error.os_code(), Some(5));
}

#[test]
fn native_error_evidence_preserves_the_failed_operation() {
    let error = OllamaFsError::from_os_code(OllamaFsErrorKind::PermissionDenied, 5)
        .at(OllamaFsOperation::OpenChild);

    assert_eq!(error.os_code(), Some(5));
    assert_eq!(error.operation(), Some(OllamaFsOperation::OpenChild));
}

#[test]
fn windows_verified_delete_contract_forbids_path_recursive_fallback() {
    let source = include_str!("durable_fs_windows.rs");
    let verified = include_str!("durable_fs_windows_verified.rs");
    let entries = include_str!("durable_fs_windows_verified/entries.rs");
    let method = source
        .split_once("fn remove_tree_verified")
        .and_then(|(_, remainder)| remainder.split_once("\n    fn sync_file"))
        .map(|(body, _)| body)
        .expect("verified Windows deletion method");

    assert!(!method.contains("remove_tree(root.path())"));
    assert!(method.contains("verified::remove_tree(root)"));
    assert!(verified.contains("use super::super::OllamaFsOperation;"));
    assert!(entries.contains("super::super::super::{"));
    assert!(entries.contains("OllamaFsOperation"));
}

#[test]
fn windows_verified_delete_opens_paths_then_revalidates_native_identity() {
    let verified = include_str!("durable_fs_windows_verified.rs");
    let handles = include_str!("durable_fs_windows_verified/handles.rs");
    let entries = include_str!("durable_fs_windows_verified/entries.rs");

    assert!(verified.contains("handles::open_path(root.path(), true)"));
    assert!(entries.contains("handles::open_path(&child_path, entry.directory)"));
    assert!(
        entries.contains("handles::matches_identity(&info, entry.file_id as u64, volume_serial)")
    );
    assert!(handles.contains("CreateFileW"));
    assert!(!handles.contains("OpenFileById"));
    assert!(!handles.contains("ReOpenFile"));
}

#[test]
fn wide_path_validation_rejects_nul_and_32768_units() {
    assert_eq!(
        validate_wide_units([b'a' as u16, 0, b'b' as u16]),
        Err(OllamaFsErrorKind::InvalidInput)
    );
    assert_eq!(
        validate_wide_units(std::iter::repeat_n(b'a' as u16, 32_768)),
        Err(OllamaFsErrorKind::InvalidInput)
    );
    assert!(validate_wide_units(std::iter::repeat_n(b'a' as u16, 32_767)).is_ok());
}

#[cfg(unix)]
#[test]
fn unix_atomic_primitive_distinguishes_new_and_replace() {
    let root = tempfile::tempdir().unwrap();
    let fs = PlatformOllamaDurableFs;
    let tmp = root.path().join("journal.tmp");
    let replacement_tmp = root.path().join("replacement.tmp");
    let final_path = root.path().join("journal.json");

    fs.write_new_atomic(&tmp, &final_path, b"first").unwrap();
    assert_eq!(std::fs::read(&final_path).unwrap(), b"first");
    let error = fs
        .write_new_atomic(&tmp, &final_path, b"second")
        .unwrap_err();
    assert_eq!(error.kind(), OllamaFsErrorKind::AlreadyExists);
    assert_eq!(std::fs::read(&final_path).unwrap(), b"first");
    assert!(tmp.exists());

    fs.replace_atomic(&replacement_tmp, &final_path, b"second")
        .unwrap();
    assert_eq!(std::fs::read(&final_path).unwrap(), b"second");
    fs.remove_file_durable(&final_path).unwrap();
    assert!(!final_path.exists());
}
