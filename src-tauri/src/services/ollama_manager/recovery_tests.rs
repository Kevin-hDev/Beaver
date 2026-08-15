use super::cleanup;
use super::durable_fs::{OllamaDurableFs, OllamaFsError, OllamaFsErrorKind};
use super::error::OllamaErrorCode;
use super::fingerprint::{BundleFingerprint, OllamaVersion, Sha256Digest};
use super::journal::{OllamaJournalState, OllamaTransactionJournal};
use super::recovery::{RecoveryExecutor, RecoveryProbe, RecoveryProbeResult, RecoveryReason};
use super::recovery_decision::{
    DirectoryEvidence, JournalPresence, MigrationMarkerPresence, OllamaLayoutSnapshot,
};
use crate::services::paths::{ollama_paths, OllamaPaths};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Failure {
    BeforeRename,
    AfterRename,
    BeforeRemove,
    AfterRemove,
    BeforeSync,
    AfterSync,
    BeforeWrite,
    AfterWrite,
    BeforePublish,
    AfterPublish,
}

#[derive(Default)]
struct CutpointFs {
    calls: Mutex<Vec<&'static str>>,
    failure: Mutex<Option<Failure>>,
    layout: Mutex<HashSet<PathBuf>>,
    final_bytes: Mutex<Option<Vec<u8>>>,
}

impl CutpointFs {
    fn seed(&self, path: &Path) {
        std::fs::create_dir_all(path).unwrap();
        self.layout.lock().unwrap().insert(path.to_path_buf());
    }
    fn has(&self, path: &Path) -> bool {
        self.layout.lock().unwrap().contains(path)
    }
    fn fail_at(&self, point: Failure) {
        *self.failure.lock().unwrap() = Some(point);
    }
    fn clear_failure(&self) {
        *self.failure.lock().unwrap() = None;
    }
    fn calls(&self) -> Vec<&'static str> {
        self.calls.lock().unwrap().clone()
    }
    fn clear_calls(&self) {
        self.calls.lock().unwrap().clear();
    }
    fn event(&self, value: &'static str) {
        self.calls.lock().unwrap().push(value);
    }
    fn fail(&self, point: Failure) -> Result<(), OllamaFsError> {
        if *self.failure.lock().unwrap() == Some(point) {
            Err(OllamaFsError::new(OllamaFsErrorKind::Other))
        } else {
            Ok(())
        }
    }
}

impl OllamaDurableFs for CutpointFs {
    fn read_bounded(&self, path: &Path, _max: usize) -> Result<Vec<u8>, OllamaFsError> {
        if path.extension().and_then(|value| value.to_str()) == Some("tmp") {
            return Err(OllamaFsError::new(OllamaFsErrorKind::NotFound));
        }
        self.final_bytes
            .lock()
            .unwrap()
            .clone()
            .ok_or_else(|| OllamaFsError::new(OllamaFsErrorKind::NotFound))
    }
    fn create_directory_durable(&self, _path: &Path) -> Result<(), OllamaFsError> {
        self.event("mkdir");
        Ok(())
    }
    fn write_new_atomic(
        &self,
        _tmp: &Path,
        _final: &Path,
        bytes: &[u8],
    ) -> Result<(), OllamaFsError> {
        self.event("journal_write");
        self.fail(Failure::BeforeWrite)?;
        *self.final_bytes.lock().unwrap() = Some(bytes.to_vec());
        self.fail(Failure::AfterWrite)?;
        self.event("journal_sync");
        self.fail(Failure::BeforeSync)?;
        self.fail(Failure::AfterSync)?;
        self.event("journal_rename");
        self.fail(Failure::BeforePublish)?;
        self.fail(Failure::AfterPublish)?;
        Ok(())
    }
    fn replace_atomic(
        &self,
        tmp: &Path,
        final_path: &Path,
        bytes: &[u8],
    ) -> Result<(), OllamaFsError> {
        self.write_new_atomic(tmp, final_path, bytes)
    }
    fn rename_durable(&self, source: &Path, destination: &Path) -> Result<(), OllamaFsError> {
        self.event("layout_rename");
        self.fail(Failure::BeforeRename)?;
        let mut layout = self.layout.lock().unwrap();
        layout.remove(source);
        layout.insert(destination.to_path_buf());
        drop(layout);
        let _ = std::fs::create_dir_all(destination);
        self.fail(Failure::AfterRename)?;
        self.event("layout_sync");
        self.fail(Failure::BeforeSync)?;
        self.fail(Failure::AfterSync)?;
        Ok(())
    }
    fn remove_file_durable(&self, _path: &Path) -> Result<(), OllamaFsError> {
        self.event("file_remove");
        self.fail(Failure::BeforeRemove)?;
        self.final_bytes.lock().unwrap().take();
        self.fail(Failure::AfterRemove)?;
        self.event("file_sync");
        self.fail(Failure::BeforeSync)?;
        self.fail(Failure::AfterSync)?;
        Ok(())
    }
    fn remove_tree(&self, path: &Path) -> Result<(), OllamaFsError> {
        self.event("layout_remove");
        self.fail(Failure::BeforeRemove)?;
        self.layout.lock().unwrap().remove(path);
        self.fail(Failure::AfterRemove)?;
        self.event("layout_sync");
        self.fail(Failure::BeforeSync)?;
        self.fail(Failure::AfterSync)?;
        Ok(())
    }
    fn sync_file(&self, _path: &Path) -> Result<(), OllamaFsError> {
        self.event("sync_file");
        self.fail(Failure::BeforeSync)?;
        self.fail(Failure::AfterSync)
    }
    fn sync_parent(&self, _path: &Path) -> Result<(), OllamaFsError> {
        self.event("sync_parent");
        self.fail(Failure::BeforeSync)?;
        self.fail(Failure::AfterSync)
    }
}

struct ValidProbe;
impl RecoveryProbe for ValidProbe {
    fn validate(&self, _target: &BundleFingerprint) -> RecoveryProbeResult {
        RecoveryProbeResult::Valid
    }
}

fn fp(version: &str, byte: &str) -> BundleFingerprint {
    BundleFingerprint {
        version: OllamaVersion::parse(version).unwrap(),
        executable_sha256: Sha256Digest::from_hex(&byte.repeat(32)).unwrap(),
    }
}
fn known(version: &str, byte: &str) -> DirectoryEvidence {
    DirectoryEvidence::Present(fp(version, byte))
}
fn empty() -> OllamaLayoutSnapshot {
    OllamaLayoutSnapshot {
        journal: JournalPresence::Absent,
        migration_marker: MigrationMarkerPresence::Absent,
        active: DirectoryEvidence::Absent,
        install_staging: DirectoryEvidence::Absent,
        update_staging: DirectoryEvidence::Absent,
        backup: DirectoryEvidence::Absent,
        failed: DirectoryEvidence::Absent,
        legacy_staging: DirectoryEvidence::Absent,
        legacy_backup: DirectoryEvidence::Absent,
        backup_delete: DirectoryEvidence::Absent,
        failed_delete: DirectoryEvidence::Absent,
    }
}
fn setup() -> (
    tempfile::TempDir,
    Arc<CutpointFs>,
    RecoveryExecutor<CutpointFs, ValidProbe>,
    OllamaPaths,
) {
    let root = tempfile::tempdir_in(".").unwrap();
    let paths = ollama_paths(root.path());
    let fs = Arc::new(CutpointFs::default());
    let runner = RecoveryExecutor::new(Arc::clone(&fs), Arc::new(ValidProbe), paths.clone());
    (root, fs, runner, paths)
}
fn cleanup_snapshot(fs: &CutpointFs, paths: &OllamaPaths) -> OllamaLayoutSnapshot {
    let mut snapshot = empty();
    snapshot.journal = JournalPresence::Valid(OllamaTransactionJournal::new(
        OllamaJournalState::CleanupPending {
            target: fp("1.2.3", "11"),
            previous: fp("1.2.2", "22"),
        },
    ));
    snapshot.active = known("1.2.3", "11");
    if fs.has(&paths.backup) {
        snapshot.backup = known("1.2.2", "22");
    }
    if fs.has(&paths.backup_delete) {
        snapshot.backup_delete = known("1.2.2", "22");
    }
    snapshot
}
fn rollback_snapshot(fs: &CutpointFs, paths: &OllamaPaths) -> OllamaLayoutSnapshot {
    let mut snapshot = empty();
    snapshot.journal = JournalPresence::Valid(OllamaTransactionJournal::new(
        OllamaJournalState::RollbackPending {
            previous: fp("1.2.2", "22"),
            rejected_target: Some(fp("1.2.3", "33")),
        },
    ));
    if fs.has(&paths.active) {
        let active_byte = if fs.has(&paths.failed) || fs.has(&paths.failed_delete) {
            "22"
        } else {
            "33"
        };
        snapshot.active = known(
            if active_byte == "22" {
                "1.2.2"
            } else {
                "1.2.3"
            },
            active_byte,
        );
    }
    if fs.has(&paths.backup) {
        snapshot.backup = known("1.2.2", "22");
    }
    if fs.has(&paths.failed) {
        snapshot.failed = known("1.2.3", "33");
    }
    if fs.has(&paths.failed_delete) {
        snapshot.failed_delete = known("1.2.3", "33");
    }
    snapshot
}

#[tokio::test]
async fn cleanup_rename_and_sync_cutpoints_resume_without_loss() {
    for failure in [
        Failure::BeforeRename,
        Failure::AfterRename,
        Failure::BeforeSync,
        Failure::AfterSync,
    ] {
        let (_root, fs, runner, paths) = setup();
        fs.seed(&paths.active);
        fs.seed(&paths.backup);
        fs.fail_at(failure);
        let first = runner
            .execute_snapshot(&cleanup_snapshot(&fs, &paths), RecoveryReason::Startup)
            .await;
        assert_eq!(first, Err(OllamaErrorCode::OllamaStorageUnavailable));
        fs.clear_failure();
        for _ in 0..2 {
            let _ = runner
                .execute_snapshot(&cleanup_snapshot(&fs, &paths), RecoveryReason::Retry)
                .await;
        }
        assert!(!(fs.has(&paths.backup) && fs.has(&paths.backup_delete)));
        assert!(fs.has(&paths.active));
    }
}

#[tokio::test]
async fn cleanup_remove_cutpoints_resume_from_the_same_rebut() {
    for failure in [
        Failure::BeforeRemove,
        Failure::AfterRemove,
        Failure::BeforeSync,
        Failure::AfterSync,
    ] {
        let (_root, fs, runner, paths) = setup();
        fs.seed(&paths.active);
        fs.seed(&paths.backup_delete);
        fs.fail_at(failure);
        let first = runner
            .execute_snapshot(&cleanup_snapshot(&fs, &paths), RecoveryReason::Retry)
            .await;
        assert_eq!(first, Err(OllamaErrorCode::OllamaStorageUnavailable));
        fs.clear_failure();
        for _ in 0..2 {
            let _ = runner
                .execute_snapshot(&cleanup_snapshot(&fs, &paths), RecoveryReason::Retry)
                .await;
        }
        assert!(fs.has(&paths.active));
        assert!(!fs.has(&paths.backup));
    }
}

#[tokio::test]
async fn rollback_cutpoints_keep_rejected_target_until_cleanup_finishes() {
    for failure in [
        Failure::BeforeRename,
        Failure::AfterRename,
        Failure::BeforeSync,
        Failure::AfterSync,
    ] {
        let (_root, fs, runner, paths) = setup();
        fs.seed(&paths.active);
        fs.seed(&paths.backup);
        fs.fail_at(failure);
        let first = runner
            .execute_snapshot(&rollback_snapshot(&fs, &paths), RecoveryReason::Startup)
            .await;
        assert_eq!(first, Err(OllamaErrorCode::OllamaStorageUnavailable));
        fs.clear_failure();
        fs.clear_calls();
        let mut snapshot = rollback_snapshot(&fs, &paths);
        for _ in 0..6 {
            let _ = runner
                .execute_snapshot(&snapshot, RecoveryReason::Retry)
                .await;
            snapshot = rollback_snapshot(&fs, &paths);
            if fs.final_bytes.lock().unwrap().is_some() {
                let journal = OllamaTransactionJournal::parse_bounded(
                    &fs.final_bytes.lock().unwrap().clone().unwrap(),
                )
                .unwrap();
                snapshot.journal = JournalPresence::Valid(journal);
            }
        }
        assert!(fs.has(&paths.active));
        assert!(!fs.has(&paths.backup));
        assert!(!fs.has(&paths.failed) || fs.has(&paths.failed_delete));
    }
}

#[tokio::test]
async fn migration_marker_cutpoints_retry_without_layout_deletion() {
    for failure in [
        Failure::BeforeWrite,
        Failure::AfterWrite,
        Failure::BeforeSync,
        Failure::AfterSync,
        Failure::BeforePublish,
        Failure::AfterPublish,
    ] {
        let (_root, fs, runner, paths) = setup();
        fs.seed(&paths.active);
        let mut snapshot = empty();
        snapshot.active = known("1.2.3", "11");
        fs.fail_at(failure);
        assert_eq!(
            runner
                .execute_snapshot(&snapshot, RecoveryReason::Startup)
                .await,
            Err(OllamaErrorCode::OllamaStorageUnavailable)
        );
        fs.clear_failure();
        snapshot.migration_marker = if fs.final_bytes.lock().unwrap().is_some() {
            MigrationMarkerPresence::Valid(Default::default())
        } else {
            MigrationMarkerPresence::Absent
        };
        for _ in 0..2 {
            let _ = runner
                .execute_snapshot(&snapshot, RecoveryReason::Retry)
                .await;
            snapshot.migration_marker = MigrationMarkerPresence::Valid(Default::default());
        }
        assert!(fs.has(&paths.active));
        assert!(!fs.calls().contains(&"layout_remove"));
    }
}

#[tokio::test]
async fn failed_backup_cleanup_does_not_remove_or_alias_failed_rebut() {
    let (_root, fs, runner, paths) = setup();
    fs.seed(&paths.active);
    fs.seed(&paths.backup_delete);
    fs.seed(&paths.failed_delete);
    fs.fail_at(Failure::BeforeRemove);
    let snapshot = cleanup_snapshot(&fs, &paths);
    assert_eq!(
        runner
            .execute_snapshot(&snapshot, RecoveryReason::Retry)
            .await,
        Err(OllamaErrorCode::OllamaStorageUnavailable)
    );
    assert!(fs.has(&paths.failed_delete));
    assert!(!fs.has(&paths.failed));
}

#[cfg(unix)]
#[tokio::test]
async fn journal_tmp_matrix_preserves_invalid_regular_and_nonregular_entries() {
    use std::os::unix::fs::symlink;
    let root = tempfile::tempdir_in(".").unwrap();
    let paths = ollama_paths(root.path());
    let fs = Arc::new(super::durable_fs::platform_fs());
    let journal = super::journal_store::OllamaJournalStore::new(Arc::clone(&fs), paths.clone());
    std::fs::write(&paths.journal_tmp, b"partial").unwrap();
    assert!(cleanup::remove_safe_journal_tmp(&fs, &journal, &paths)
        .await
        .unwrap());
    let valid = serde_json::to_vec(&OllamaTransactionJournal::new(
        OllamaJournalState::Prepared {
            target: fp("1.2.3", "11"),
            previous: fp("1.2.2", "22"),
        },
    ))
    .unwrap();
    std::fs::write(&paths.journal, &valid).unwrap();
    std::fs::write(&paths.journal_tmp, b"partial").unwrap();
    assert!(cleanup::remove_safe_journal_tmp(&fs, &journal, &paths)
        .await
        .unwrap());
    assert_eq!(std::fs::read(&paths.journal).unwrap(), valid);
    std::fs::write(&paths.journal, b"invalid").unwrap();
    std::fs::write(&paths.journal_tmp, b"partial").unwrap();
    assert_eq!(
        cleanup::remove_safe_journal_tmp(&fs, &journal, &paths).await,
        Err(OllamaErrorCode::OllamaUpdateRecoveryRequired)
    );
    assert!(paths.journal_tmp.exists());
    std::fs::remove_file(&paths.journal_tmp).unwrap();
    std::fs::create_dir(&paths.journal_tmp).unwrap();
    assert_eq!(
        cleanup::remove_safe_journal_tmp(&fs, &journal, &paths).await,
        Err(OllamaErrorCode::OllamaUpdateRecoveryRequired)
    );
    std::fs::remove_dir(&paths.journal_tmp).unwrap();
    std::fs::write(&paths.journal_tmp, vec![b'x'; 4097]).unwrap();
    assert_eq!(
        cleanup::remove_safe_journal_tmp(&fs, &journal, &paths).await,
        Err(OllamaErrorCode::OllamaUpdateRecoveryRequired)
    );
    std::fs::remove_file(&paths.journal_tmp).unwrap();
    let target = root.path().join("target");
    std::fs::write(&target, b"x").unwrap();
    symlink(&target, &paths.journal_tmp).unwrap();
    assert_eq!(
        cleanup::remove_safe_journal_tmp(&fs, &journal, &paths).await,
        Err(OllamaErrorCode::OllamaUpdateRecoveryRequired)
    );
    assert!(paths.journal_tmp.exists());
}
