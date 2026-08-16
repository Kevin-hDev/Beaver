use super::cleanup;
use super::durable_fs::{platform_fs, OllamaDurableFs, OllamaFsError, OllamaFsErrorKind};
use super::error::OllamaErrorCode;
use super::fingerprint::{BundleFingerprint, OllamaVersion, Sha256Digest};
use super::journal::{OllamaJournalState, OllamaTransactionJournal};
use super::path_identity::{CanonicalDirectory, NativePathIdentityResolver, PathIdentityResolver};
use super::recovery::{RecoveryExecutor, RecoveryProbe, RecoveryProbeResult, RecoveryReason};
use super::recovery_decision::{
    ArchiveDirectoryEvidence, DirectoryEvidence, JournalPresence, MigrationMarkerPresence,
    OllamaLayoutSnapshot,
};
use super::release_source::{OllamaArchive, OllamaReleaseManifest};
use crate::services::paths::{ollama_paths, OllamaPaths};
use sha2::Digest;
use std::collections::HashSet;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

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
    fn remove_tree_verified(&self, root: &CanonicalDirectory) -> Result<(), OllamaFsError> {
        self.remove_tree(root.path())
    }
    fn sync_parent(&self, _path: &Path) -> Result<(), OllamaFsError> {
        self.event("sync_parent");
        self.fail(Failure::BeforeSync)?;
        self.fail(Failure::AfterSync)
    }
}

struct ValidProbe;
#[async_trait::async_trait]
impl RecoveryProbe for ValidProbe {
    async fn validate(
        &self,
        _target: &BundleFingerprint,
        _paths: &OllamaPaths,
    ) -> RecoveryProbeResult {
        RecoveryProbeResult::Valid
    }
}

struct InvalidProbe;
#[async_trait::async_trait]
impl RecoveryProbe for InvalidProbe {
    async fn validate(
        &self,
        _target: &BundleFingerprint,
        _paths: &OllamaPaths,
    ) -> RecoveryProbeResult {
        RecoveryProbeResult::Invalid(OllamaErrorCode::OllamaBundleInvalid)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RealOperation {
    Rename,
    SyncFile,
    Remove,
    Write,
    Publish,
    SyncParent,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct RealCutpoint {
    operation: RealOperation,
    before: bool,
}

#[derive(Default)]
struct RealCutpointFs {
    failure: Mutex<Option<RealCutpoint>>,
    events: Mutex<Vec<RealCutpoint>>,
}

impl RealCutpointFs {
    fn fail_at(&self, cutpoint: RealCutpoint) {
        *self.failure.lock().unwrap() = Some(cutpoint);
    }

    fn clear_failure(&self) {
        *self.failure.lock().unwrap() = None;
    }

    fn boundary(&self, operation: RealOperation, before: bool) -> Result<(), OllamaFsError> {
        let cutpoint = RealCutpoint { operation, before };
        self.events.lock().unwrap().push(cutpoint);
        if *self.failure.lock().unwrap() == Some(cutpoint) {
            Err(OllamaFsError::new(OllamaFsErrorKind::Other))
        } else {
            Ok(())
        }
    }

    fn events(&self) -> Vec<RealCutpoint> {
        self.events.lock().unwrap().clone()
    }
}

impl OllamaDurableFs for RealCutpointFs {
    fn read_bounded(&self, path: &Path, max_bytes: usize) -> Result<Vec<u8>, OllamaFsError> {
        let metadata = fs::symlink_metadata(path).map_err(real_io_error)?;
        if !metadata.is_file() || metadata.len() > max_bytes as u64 {
            return Err(OllamaFsError::new(OllamaFsErrorKind::InvalidInput));
        }
        let mut bytes = Vec::new();
        File::open(path)
            .map_err(real_io_error)?
            .take(max_bytes.saturating_add(1) as u64)
            .read_to_end(&mut bytes)
            .map_err(real_io_error)?;
        if bytes.len() > max_bytes {
            return Err(OllamaFsError::new(OllamaFsErrorKind::InvalidInput));
        }
        Ok(bytes)
    }

    fn create_directory_durable(&self, path: &Path) -> Result<(), OllamaFsError> {
        platform_fs().create_directory_durable(path)
    }

    fn write_new_atomic(
        &self,
        tmp: &Path,
        final_path: &Path,
        bytes: &[u8],
    ) -> Result<(), OllamaFsError> {
        self.write_atomic(tmp, final_path, bytes, false)
    }

    fn replace_atomic(
        &self,
        tmp: &Path,
        final_path: &Path,
        bytes: &[u8],
    ) -> Result<(), OllamaFsError> {
        self.write_atomic(tmp, final_path, bytes, true)
    }

    fn rename_durable(&self, source: &Path, destination: &Path) -> Result<(), OllamaFsError> {
        self.boundary(RealOperation::Rename, true)?;
        fs::rename(source, destination).map_err(real_io_error)?;
        self.boundary(RealOperation::Rename, false)?;
        self.sync_parents(source, destination)
    }

    fn remove_file_durable(&self, path: &Path) -> Result<(), OllamaFsError> {
        self.remove_file(path)
    }

    fn remove_tree(&self, root: &Path) -> Result<(), OllamaFsError> {
        self.boundary(RealOperation::Remove, true)?;
        fs::remove_dir_all(root).map_err(real_io_error)?;
        self.boundary(RealOperation::Remove, false)?;
        self.sync_parents(root, root)
    }

    fn remove_tree_verified(&self, root: &CanonicalDirectory) -> Result<(), OllamaFsError> {
        self.remove_tree(root.path())
    }

    fn sync_file(&self, path: &Path) -> Result<(), OllamaFsError> {
        self.boundary(RealOperation::SyncFile, true)?;
        platform_fs().sync_file(path)?;
        self.boundary(RealOperation::SyncFile, false)
    }

    fn sync_parent(&self, path: &Path) -> Result<(), OllamaFsError> {
        self.boundary(RealOperation::SyncParent, true)?;
        sync_parent_path(path)?;
        self.boundary(RealOperation::SyncParent, false)
    }
}

impl RealCutpointFs {
    fn write_atomic(
        &self,
        tmp: &Path,
        final_path: &Path,
        bytes: &[u8],
        replace: bool,
    ) -> Result<(), OllamaFsError> {
        self.boundary(RealOperation::Write, true)?;
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(tmp)
            .map_err(real_io_error)?;
        file.write_all(bytes).map_err(real_io_error)?;
        self.boundary(RealOperation::Write, false)?;
        self.boundary(RealOperation::SyncFile, true)?;
        file.sync_all().map_err(real_io_error)?;
        drop(file);
        self.boundary(RealOperation::SyncFile, false)?;
        self.boundary(RealOperation::Publish, true)?;
        if !replace && final_path.exists() {
            return Err(OllamaFsError::new(OllamaFsErrorKind::AlreadyExists));
        }
        fs::rename(tmp, final_path).map_err(real_io_error)?;
        self.boundary(RealOperation::Publish, false)?;
        self.sync_parents(tmp, final_path)
    }

    fn remove_file(&self, path: &Path) -> Result<(), OllamaFsError> {
        self.boundary(RealOperation::Remove, true)?;
        fs::remove_file(path).map_err(real_io_error)?;
        self.boundary(RealOperation::Remove, false)?;
        self.sync_parents(path, path)
    }

    fn sync_parents(&self, source: &Path, destination: &Path) -> Result<(), OllamaFsError> {
        self.boundary(RealOperation::SyncParent, true)?;
        sync_parent_path(source)?;
        if source.parent() != destination.parent() {
            sync_parent_path(destination)?;
        }
        self.boundary(RealOperation::SyncParent, false)
    }
}

fn real_io_error(error: std::io::Error) -> OllamaFsError {
    let kind = match error.kind() {
        std::io::ErrorKind::NotFound => OllamaFsErrorKind::NotFound,
        std::io::ErrorKind::AlreadyExists => OllamaFsErrorKind::AlreadyExists,
        std::io::ErrorKind::InvalidInput => OllamaFsErrorKind::InvalidInput,
        _ => OllamaFsErrorKind::Other,
    };
    OllamaFsError::new(kind)
}

fn sync_parent_path(path: &Path) -> Result<(), OllamaFsError> {
    platform_fs().sync_parent(path)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RealScenario {
    RemoveUncommittedInstallStaging,
    CommitFreshInstallFromActive,
    CommitFreshInstallFromUpdate,
    RestoreLegacyBackup,
    RestoreLegacyBackupFromModernBackup,
    ResumeTargetValidation,
    PersistRollbackCleanupPending,
    MoveFailedToDelete,
    RemoveFailedDelete,
    RemoveCompletedLegacyJournal,
    AdoptLegacyActive,
    CleanupPendingMoveBackupToDelete,
    CleanupPendingRemoveBackupDelete,
    RollbackPendingMoveRejectedToFailed,
    RollbackPendingRestorePrevious,
}

struct RealFixture {
    _root: tempfile::TempDir,
    paths: OllamaPaths,
    fs: Arc<RealCutpointFs>,
    runner: RecoveryExecutor<RealCutpointFs, ValidProbe>,
}

fn real_bundle_at(path: &Path, version: &str, body: &[u8]) -> BundleFingerprint {
    let bin = path.join("bin");
    std::fs::create_dir_all(&bin).expect("bundle directory");
    std::fs::write(path.join("VERSION"), version).expect("version");
    let executable = bin.join(if cfg!(windows) {
        "ollama.exe"
    } else {
        "ollama"
    });
    std::fs::write(&executable, body).expect("executable");
    #[cfg(unix)]
    std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o755))
        .expect("executable permissions");
    let fingerprint = BundleFingerprint {
        version: OllamaVersion::parse(version).expect("version fingerprint"),
        executable_sha256: super::probe_http::hash_file(&executable)
            .ok()
            .expect("executable hash"),
    };
    super::bundle_receipt::write_receipt(
        &platform_fs(),
        path,
        &super::bundle_receipt::BundleReceipt::new(fingerprint.clone()),
    )
    .expect("bundle receipt");
    fingerprint
}

fn write_real_journal(paths: &OllamaPaths, journal: &OllamaTransactionJournal) {
    std::fs::write(
        &paths.journal,
        serde_json::to_vec(journal).expect("journal bytes"),
    )
    .expect("journal");
}

fn real_fixture(scenario: RealScenario) -> RealFixture {
    let root = tempfile::tempdir_in(".").expect("layout root");
    let paths = ollama_paths(root.path());
    let models_path = root.path().join("models");
    std::fs::create_dir_all(&models_path).expect("models directory");
    let models = NativePathIdentityResolver
        .canonical_directory(&models_path)
        .expect("models identity");

    match scenario {
        RealScenario::RemoveUncommittedInstallStaging => {
            real_bundle_at(&paths.install_staging, "1.2.3", b"install");
        }
        RealScenario::CommitFreshInstallFromActive => {
            let previous = real_bundle_at(&paths.active, "1.2.2", b"previous");
            let target = real_bundle_at(&paths.update_staging, "1.2.3", b"target");
            write_real_journal(
                &paths,
                &OllamaTransactionJournal::new(OllamaJournalState::Prepared { target, previous }),
            );
        }
        RealScenario::CommitFreshInstallFromUpdate => {
            let target = real_bundle_at(&paths.update_staging, "1.2.3", b"target");
            let previous = real_bundle_at(&paths.backup, "1.2.2", b"previous");
            write_real_journal(
                &paths,
                &OllamaTransactionJournal::new(OllamaJournalState::Prepared { target, previous }),
            );
        }
        RealScenario::RestoreLegacyBackup => {
            real_bundle_at(&paths.legacy_backup, "1.2.2", b"legacy");
        }
        RealScenario::RestoreLegacyBackupFromModernBackup => {
            real_bundle_at(&paths.backup, "1.2.2", b"modern-backup");
        }
        RealScenario::ResumeTargetValidation => {
            let target = real_bundle_at(&paths.active, "1.2.3", b"target");
            let previous = real_bundle_at(&paths.backup, "1.2.2", b"previous");
            write_real_journal(
                &paths,
                &OllamaTransactionJournal::new(OllamaJournalState::PendingValidation {
                    target,
                    previous,
                }),
            );
        }
        RealScenario::PersistRollbackCleanupPending => {
            let previous = real_bundle_at(&paths.active, "1.2.2", b"previous");
            let rejected = real_bundle_at(&paths.failed, "1.2.3", b"rejected");
            write_real_journal(
                &paths,
                &OllamaTransactionJournal::new(OllamaJournalState::RollbackPending {
                    previous,
                    rejected_target: Some(rejected),
                }),
            );
        }
        RealScenario::MoveFailedToDelete => {
            let previous = real_bundle_at(&paths.active, "1.2.2", b"previous");
            let rejected = real_bundle_at(&paths.failed, "1.2.3", b"rejected");
            write_real_journal(
                &paths,
                &OllamaTransactionJournal::new(OllamaJournalState::RollbackCleanupPending {
                    previous,
                    rejected_target: Some(rejected),
                }),
            );
        }
        RealScenario::RemoveFailedDelete => {
            let previous = real_bundle_at(&paths.active, "1.2.2", b"previous");
            let rejected = real_bundle_at(&paths.failed_delete, "1.2.3", b"rejected");
            write_real_journal(
                &paths,
                &OllamaTransactionJournal::new(OllamaJournalState::RollbackCleanupPending {
                    previous,
                    rejected_target: Some(rejected),
                }),
            );
        }
        RealScenario::RemoveCompletedLegacyJournal => {
            let target = real_bundle_at(&paths.active, "1.2.3", b"active");
            write_real_journal(
                &paths,
                &OllamaTransactionJournal::new(OllamaJournalState::CleanupPending {
                    target,
                    previous: fp("1.2.2", "22"),
                }),
            );
            std::fs::write(
                &paths.migration_marker,
                serde_json::to_vec(&super::journal::OllamaMigrationMarker::new())
                    .expect("marker bytes"),
            )
            .expect("marker");
        }
        RealScenario::AdoptLegacyActive => {
            real_bundle_at(&paths.active, "1.2.3", b"active");
        }
        RealScenario::CleanupPendingMoveBackupToDelete => {
            let target = real_bundle_at(&paths.active, "1.2.3", b"target");
            let previous = real_bundle_at(&paths.backup, "1.2.2", b"previous");
            write_real_journal(
                &paths,
                &OllamaTransactionJournal::new(OllamaJournalState::CleanupPending {
                    target,
                    previous,
                }),
            );
        }
        RealScenario::CleanupPendingRemoveBackupDelete => {
            let target = real_bundle_at(&paths.active, "1.2.3", b"target");
            let previous = real_bundle_at(&paths.backup_delete, "1.2.2", b"previous");
            write_real_journal(
                &paths,
                &OllamaTransactionJournal::new(OllamaJournalState::CleanupPending {
                    target,
                    previous,
                }),
            );
        }
        RealScenario::RollbackPendingMoveRejectedToFailed => {
            let rejected = real_bundle_at(&paths.active, "1.2.3", b"rejected");
            let previous = real_bundle_at(&paths.backup, "1.2.2", b"previous");
            write_real_journal(
                &paths,
                &OllamaTransactionJournal::new(OllamaJournalState::RollbackPending {
                    previous,
                    rejected_target: Some(rejected),
                }),
            );
        }
        RealScenario::RollbackPendingRestorePrevious => {
            let previous = real_bundle_at(&paths.backup, "1.2.2", b"previous");
            write_real_journal(
                &paths,
                &OllamaTransactionJournal::new(OllamaJournalState::RollbackPending {
                    previous,
                    rejected_target: None,
                }),
            );
        }
    }

    let fs = Arc::new(RealCutpointFs::default());
    let runner = RecoveryExecutor::new_with_models(
        Arc::clone(&fs),
        Arc::new(ValidProbe),
        paths.clone(),
        models,
    );
    RealFixture {
        _root: root,
        paths,
        fs,
        runner,
    }
}

fn real_cutpoint_operations(scenario: RealScenario) -> &'static [RealOperation] {
    match scenario {
        RealScenario::RemoveUncommittedInstallStaging
        | RealScenario::CommitFreshInstallFromActive
        | RealScenario::CommitFreshInstallFromUpdate
        | RealScenario::RestoreLegacyBackup
        | RealScenario::MoveFailedToDelete => &[RealOperation::Rename, RealOperation::SyncParent],
        RealScenario::RestoreLegacyBackupFromModernBackup => {
            &[RealOperation::Rename, RealOperation::SyncParent]
        }
        RealScenario::ResumeTargetValidation | RealScenario::PersistRollbackCleanupPending => &[
            RealOperation::Write,
            RealOperation::SyncFile,
            RealOperation::Publish,
            RealOperation::SyncParent,
        ],
        RealScenario::RemoveFailedDelete => &[RealOperation::Remove, RealOperation::SyncParent],
        RealScenario::RemoveCompletedLegacyJournal => {
            &[RealOperation::Remove, RealOperation::SyncParent]
        }
        RealScenario::AdoptLegacyActive => &[
            RealOperation::Write,
            RealOperation::SyncFile,
            RealOperation::Publish,
            RealOperation::SyncParent,
        ],
        RealScenario::CleanupPendingMoveBackupToDelete => {
            &[RealOperation::Rename, RealOperation::SyncParent]
        }
        RealScenario::CleanupPendingRemoveBackupDelete => {
            &[RealOperation::Remove, RealOperation::SyncParent]
        }
        RealScenario::RollbackPendingMoveRejectedToFailed
        | RealScenario::RollbackPendingRestorePrevious => {
            &[RealOperation::Rename, RealOperation::SyncParent]
        }
    }
}

fn real_snapshot(fs: &RealCutpointFs, paths: &OllamaPaths) -> OllamaLayoutSnapshot {
    let journal = match std::fs::read(&paths.journal) {
        Ok(bytes) => OllamaTransactionJournal::parse_bounded(&bytes)
            .map(JournalPresence::Valid)
            .unwrap_or(JournalPresence::Invalid),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => JournalPresence::Absent,
        Err(_) => JournalPresence::Unknown,
    };
    cleanup::snapshot(journal, fs, paths)
}

#[tokio::test]
async fn real_recovery_decision_cutpoints_converge_after_two_passes() {
    let scenarios = [
        RealScenario::RemoveUncommittedInstallStaging,
        RealScenario::CommitFreshInstallFromActive,
        RealScenario::CommitFreshInstallFromUpdate,
        RealScenario::RestoreLegacyBackup,
        RealScenario::RestoreLegacyBackupFromModernBackup,
        RealScenario::ResumeTargetValidation,
        RealScenario::PersistRollbackCleanupPending,
        RealScenario::MoveFailedToDelete,
        RealScenario::RemoveFailedDelete,
        RealScenario::RemoveCompletedLegacyJournal,
        RealScenario::AdoptLegacyActive,
        RealScenario::CleanupPendingMoveBackupToDelete,
        RealScenario::CleanupPendingRemoveBackupDelete,
        RealScenario::RollbackPendingMoveRejectedToFailed,
        RealScenario::RollbackPendingRestorePrevious,
    ];
    for scenario in scenarios {
        for operation in real_cutpoint_operations(scenario) {
            for before in [true, false] {
                let fixture = real_fixture(scenario);
                fixture.fs.fail_at(RealCutpoint {
                    operation: *operation,
                    before,
                });
                let first = fixture.runner.recover(RecoveryReason::Startup).await;
                assert_eq!(
                    first,
                    Err(OllamaErrorCode::OllamaStorageUnavailable),
                    "first pass must fail closed at {scenario:?} {operation:?} before={before}"
                );
                fixture.fs.clear_failure();
                let mut previous = None;
                let mut converged = false;
                for _ in 0..10 {
                    let _ = fixture.runner.recover(RecoveryReason::Retry).await;
                    let current = real_snapshot(&fixture.fs, &fixture.paths);
                    if previous.as_ref() == Some(&current) {
                        converged = true;
                        break;
                    }
                    previous = Some(current);
                }
                assert!(
                    converged,
                    "two retry passes must converge at {scenario:?} {operation:?} before={before}"
                );
                assert!(
                    fixture.fs.events().contains(&RealCutpoint {
                        operation: *operation,
                        before
                    }),
                    "cutpoint was not exercised at {scenario:?} {operation:?} before={before}"
                );
            }
        }
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
        archive_staging: ArchiveDirectoryEvidence::Absent,
        archive_failed: ArchiveDirectoryEvidence::Absent,
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
    let models_path = root.path().join("models");
    std::fs::create_dir_all(&models_path).expect("models directory");
    let models = NativePathIdentityResolver
        .canonical_directory(&models_path)
        .expect("models identity");
    let runner = RecoveryExecutor::new_with_models(
        Arc::clone(&fs),
        Arc::new(ValidProbe),
        paths.clone(),
        models,
    );
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

fn real_bundle(root: &Path, name: &str, version: &str, body: &[u8]) -> BundleFingerprint {
    let bundle = root.join(name);
    let bin = bundle.join("bin");
    std::fs::create_dir_all(&bin).expect("bundle directory");
    std::fs::write(bundle.join("VERSION"), version).expect("version");
    let executable = bin.join(if cfg!(windows) {
        "ollama.exe"
    } else {
        "ollama"
    });
    std::fs::write(&executable, body).expect("executable");
    #[cfg(unix)]
    std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o755))
        .expect("executable permissions");
    let fingerprint = BundleFingerprint {
        version: OllamaVersion::parse(version).expect("version fingerprint"),
        executable_sha256: super::probe_http::hash_file(&executable)
            .ok()
            .expect("executable hash"),
    };
    super::bundle_receipt::write_receipt(
        &platform_fs(),
        &bundle,
        &super::bundle_receipt::BundleReceipt::new(fingerprint.clone()),
    )
    .expect("bundle receipt");
    fingerprint
}

#[test]
fn real_layout_snapshot_records_the_bundle_fingerprint() {
    let root = tempfile::tempdir_in(".").expect("layout root");
    let paths = ollama_paths(root.path());
    let fingerprint = real_bundle(root.path(), "ollama-bundle", "1.2.3", b"bundle");
    let fs = super::durable_fs::platform_fs();
    let snapshot = cleanup::snapshot(JournalPresence::Absent, &fs, &paths);

    assert_eq!(snapshot.active, DirectoryEvidence::Present(fingerprint));
}

#[tokio::test]
async fn real_recovery_executes_install_staging_and_reinspects_it() {
    let root = tempfile::tempdir_in(".").expect("layout root");
    let paths = ollama_paths(root.path());
    let _fingerprint = real_bundle(
        root.path(),
        "ollama-bundle-install-staging",
        "1.2.3",
        b"staging",
    );
    let models_path = root.path().join("models");
    std::fs::create_dir_all(&models_path).expect("models directory");
    let models = NativePathIdentityResolver
        .canonical_directory(&models_path)
        .expect("models identity");
    let fs = Arc::new(super::durable_fs::platform_fs());
    let runner = RecoveryExecutor::new_with_models(
        Arc::clone(&fs),
        Arc::new(ValidProbe),
        paths.clone(),
        models,
    );

    assert_eq!(
        runner.recover(RecoveryReason::Startup).await,
        Ok(super::recovery::RecoveryOutcome::ProgressMade)
    );
    assert!(!paths.install_staging.exists());
    assert!(paths.uncommitted_staging_delete.exists());
    assert_eq!(
        runner.recover(RecoveryReason::Retry).await,
        Ok(super::recovery::RecoveryOutcome::ProgressMade)
    );
    assert!(!paths.uncommitted_staging_delete.exists());
    assert_eq!(
        runner.recover(RecoveryReason::Retry).await,
        Ok(super::recovery::RecoveryOutcome::Ready)
    );
}

#[tokio::test]
async fn partial_uncommitted_stagings_converge_through_one_dedicated_trash() {
    for source_kind in 0..3 {
        let root = tempfile::tempdir_in(".").expect("layout root");
        let paths = ollama_paths(root.path());
        let source = match source_kind {
            0 => paths.install_staging.clone(),
            1 => paths.update_staging.clone(),
            _ => paths.legacy_staging.clone(),
        };
        std::fs::create_dir_all(&source).expect("partial staging");
        std::fs::write(source.join("partial.download"), b"partial").expect("partial bytes");
        let models_path = root.path().join("models");
        std::fs::create_dir_all(&models_path).expect("models directory");
        let models = NativePathIdentityResolver
            .canonical_directory(&models_path)
            .expect("models identity");
        let fs = Arc::new(super::durable_fs::platform_fs());
        let runner = RecoveryExecutor::new_with_models(
            Arc::clone(&fs),
            Arc::new(ValidProbe),
            paths.clone(),
            models,
        );

        assert_eq!(
            runner.recover(RecoveryReason::Startup).await,
            Ok(super::recovery::RecoveryOutcome::ProgressMade)
        );
        assert!(!source.exists());
        assert!(paths.uncommitted_staging_delete.exists());
        assert!(!paths.failed.exists());
        assert_eq!(
            runner.recover(RecoveryReason::Retry).await,
            Ok(super::recovery::RecoveryOutcome::ProgressMade)
        );
        assert!(!paths.uncommitted_staging_delete.exists());
        assert_eq!(
            runner.recover(RecoveryReason::Retry).await,
            Ok(super::recovery::RecoveryOutcome::Ready)
        );
    }
}

#[tokio::test]
async fn uncommitted_staging_source_and_trash_are_preserved_as_ambiguous() {
    let root = tempfile::tempdir_in(".").expect("layout root");
    let paths = ollama_paths(root.path());
    std::fs::create_dir_all(&paths.install_staging).expect("staging");
    std::fs::create_dir_all(&paths.uncommitted_staging_delete).expect("staging trash");
    let models_path = root.path().join("models");
    std::fs::create_dir_all(&models_path).expect("models directory");
    let models = NativePathIdentityResolver
        .canonical_directory(&models_path)
        .expect("models identity");
    let fs = Arc::new(super::durable_fs::platform_fs());
    let runner = RecoveryExecutor::new_with_models(
        Arc::clone(&fs),
        Arc::new(ValidProbe),
        paths.clone(),
        models,
    );

    assert_eq!(
        runner.recover(RecoveryReason::Startup).await,
        Err(OllamaErrorCode::OllamaUpdateRecoveryRequired)
    );
    assert!(paths.install_staging.exists());
    assert!(paths.uncommitted_staging_delete.exists());
}

#[tokio::test]
async fn migration_marker_temporary_is_removed_then_recreated_durably() {
    let root = tempfile::tempdir_in(".").expect("layout root");
    let paths = ollama_paths(root.path());
    real_bundle(root.path(), "ollama-bundle", "1.2.3", b"active");
    std::fs::write(
        &paths.migration_marker_tmp,
        serde_json::to_vec(&super::journal::OllamaMigrationMarker::new()).expect("marker"),
    )
    .expect("marker tmp");
    let fs = Arc::new(super::durable_fs::platform_fs());
    let runner = RecoveryExecutor::new(Arc::clone(&fs), Arc::new(ValidProbe), paths.clone());

    assert_eq!(
        runner.recover(RecoveryReason::Startup).await,
        Ok(super::recovery::RecoveryOutcome::ProgressMade)
    );
    assert!(!paths.migration_marker_tmp.exists());
    assert!(paths.active.exists());
    assert_eq!(
        runner.recover(RecoveryReason::Retry).await,
        Ok(super::recovery::RecoveryOutcome::ProgressMade)
    );
    assert!(paths.migration_marker.exists());
    assert_eq!(
        runner.recover(RecoveryReason::Retry).await,
        Ok(super::recovery::RecoveryOutcome::Ready)
    );
}

#[tokio::test]
async fn legacy_adoption_without_an_exact_bundle_receipt_is_preserved_and_blocked() {
    let root = tempfile::tempdir_in(".").expect("layout root");
    let paths = ollama_paths(root.path());
    real_bundle(root.path(), "ollama-bundle", "1.2.3", b"active");
    std::fs::remove_file(crate::services::paths::bundle_receipt_path(&paths.active))
        .expect("remove bundle receipt");
    let fs = Arc::new(platform_fs());
    let runner = RecoveryExecutor::new(Arc::clone(&fs), Arc::new(ValidProbe), paths.clone());

    assert_eq!(
        runner.recover(RecoveryReason::Startup).await,
        Err(OllamaErrorCode::OllamaUpdateRecoveryRequired)
    );
    assert!(paths.active.exists());
    assert!(!paths.migration_marker.exists());
}

#[tokio::test]
async fn interrupted_archive_staging_is_moved_to_rebut_before_retry() {
    let root = tempfile::tempdir_in(".").expect("layout root");
    let paths = ollama_paths(root.path());
    let archive_staging = super::install::archive_staging_path(&paths);
    std::fs::create_dir_all(archive_staging).expect("archive staging");
    std::fs::write(archive_staging.join("ollama-darwin.tgz"), b"partial").expect("archive");
    let models_path = root.path().join("models");
    std::fs::create_dir_all(&models_path).expect("models directory");
    let models = NativePathIdentityResolver
        .canonical_directory(&models_path)
        .expect("models identity");
    let fs = Arc::new(super::durable_fs::platform_fs());
    let runner = RecoveryExecutor::new_with_models(
        Arc::clone(&fs),
        Arc::new(ValidProbe),
        paths.clone(),
        models,
    );

    assert_eq!(
        runner.recover(RecoveryReason::Startup).await,
        Ok(super::recovery::RecoveryOutcome::ProgressMade)
    );
    assert!(!archive_staging.exists());
    assert!(paths.archive_failed.exists());
    assert_eq!(
        runner.recover(RecoveryReason::Retry).await,
        Ok(super::recovery::RecoveryOutcome::ProgressMade)
    );
    assert!(!paths.archive_staging.exists());
    assert!(!paths.archive_failed.exists());
    assert_eq!(
        runner.recover(RecoveryReason::Retry).await,
        Ok(super::recovery::RecoveryOutcome::Ready)
    );

    let archive = root.path().join("next-install.tgz");
    std::fs::write(&archive, b"not an archive").expect("next archive");
    let digest = hex::encode(sha2::Sha256::digest(b"not an archive"));
    let manifest = OllamaReleaseManifest::for_test(
        super::fingerprint::OllamaVersion::parse("1.2.3").expect("version"),
        vec![OllamaArchive::for_test(
            "ollama-darwin.tgz",
            "https://github.com/ollama/ollama/releases/download/v1.2.3/ollama-darwin.tgz",
            14,
            &digest,
        )],
    );
    let mut request = super::install::InstallRequest::for_test(root.path().to_path_buf());
    request.version = Some(super::fingerprint::OllamaVersion::parse("1.2.3").unwrap());
    request.manifest = Some(manifest);
    request.local_archives = Some(vec![archive]);
    assert_eq!(
        super::install::install(request.clone()).await,
        Err(OllamaErrorCode::OllamaBundleInvalid)
    );
    assert!(request.paths.archive_staging.exists());
}

#[tokio::test]
async fn archive_staging_ambiguity_requires_intervention_without_deletion() {
    let root = tempfile::tempdir_in(".").expect("layout root");
    let paths = ollama_paths(root.path());
    std::fs::create_dir_all(&paths.archive_staging).expect("archive staging");
    std::fs::create_dir_all(&paths.archive_failed).expect("archive rebut");
    let models_path = root.path().join("models");
    std::fs::create_dir_all(&models_path).expect("models directory");
    let models = NativePathIdentityResolver
        .canonical_directory(&models_path)
        .expect("models identity");
    let fs = Arc::new(super::durable_fs::platform_fs());
    let runner = RecoveryExecutor::new_with_models(
        Arc::clone(&fs),
        Arc::new(ValidProbe),
        paths.clone(),
        models,
    );

    assert_eq!(
        runner.recover(RecoveryReason::Startup).await,
        Err(OllamaErrorCode::OllamaUpdateRecoveryRequired)
    );
    assert!(paths.archive_staging.exists());
    assert!(paths.archive_failed.exists());
}

#[cfg(unix)]
#[tokio::test]
async fn archive_staging_symlink_requires_intervention_without_touching_target() {
    use std::os::unix::fs::symlink;

    let root = tempfile::tempdir_in(".").expect("layout root");
    let paths = ollama_paths(root.path());
    let outside = tempfile::tempdir_in(".").expect("outside");
    std::fs::write(outside.path().join("archive"), b"must survive").expect("outside archive");
    symlink(outside.path(), &paths.archive_staging).expect("archive staging alias");
    let models_path = root.path().join("models");
    std::fs::create_dir_all(&models_path).expect("models directory");
    let models = NativePathIdentityResolver
        .canonical_directory(&models_path)
        .expect("models identity");
    let fs = Arc::new(super::durable_fs::platform_fs());
    let runner = RecoveryExecutor::new_with_models(
        Arc::clone(&fs),
        Arc::new(ValidProbe),
        paths.clone(),
        models,
    );

    assert_eq!(
        runner.recover(RecoveryReason::Startup).await,
        Err(OllamaErrorCode::OllamaUpdateRecoveryRequired)
    );
    assert!(paths.archive_staging.exists());
    assert!(outside.path().join("archive").exists());
}

#[tokio::test]
async fn real_cleanup_removes_only_the_frozen_backup_rebut_then_journal() {
    let root = tempfile::tempdir_in(".").expect("layout root");
    let paths = ollama_paths(root.path());
    let active = real_bundle(root.path(), "ollama-bundle", "1.2.3", b"active");
    let previous = real_bundle(
        root.path(),
        "ollama-bundle-backup-delete",
        "1.2.2",
        b"previous",
    );
    let journal = OllamaTransactionJournal::new(OllamaJournalState::CleanupPending {
        target: active,
        previous,
    });
    std::fs::write(
        &paths.journal,
        serde_json::to_vec(&journal).expect("journal"),
    )
    .expect("journal");
    let models_path = root.path().join("models");
    std::fs::create_dir_all(&models_path).expect("models directory");
    let models = NativePathIdentityResolver
        .canonical_directory(&models_path)
        .expect("models identity");
    let fs = Arc::new(super::durable_fs::platform_fs());
    let runner = RecoveryExecutor::new_with_models(
        Arc::clone(&fs),
        Arc::new(ValidProbe),
        paths.clone(),
        models,
    );

    assert_eq!(
        runner.recover(RecoveryReason::Startup).await,
        Ok(super::recovery::RecoveryOutcome::ProgressMade)
    );
    assert!(!paths.backup_delete.exists());
    assert!(paths.active.exists());
    assert_eq!(
        runner.recover(RecoveryReason::Retry).await,
        Ok(super::recovery::RecoveryOutcome::ProgressMade)
    );
    assert!(!paths.journal.exists());
    assert_eq!(
        runner.recover(RecoveryReason::Retry).await,
        Ok(super::recovery::RecoveryOutcome::ProgressMade)
    );
    assert!(paths.migration_marker.exists());
    assert_eq!(
        runner.recover(RecoveryReason::Retry).await,
        Ok(super::recovery::RecoveryOutcome::Ready)
    );
}

#[tokio::test]
async fn partially_deleted_rebuts_finish_without_recomputing_their_fingerprint() {
    for rollback_cleanup in [false, true] {
        let root = tempfile::tempdir_in(".").expect("layout root");
        let paths = ollama_paths(root.path());
        let active = real_bundle(root.path(), "ollama-bundle", "1.2.3", b"active");
        let deleted_fingerprint = fp("1.2.2", "22");
        let trash = if rollback_cleanup {
            &paths.failed_delete
        } else {
            &paths.backup_delete
        };
        std::fs::create_dir_all(trash).expect("partial trash");
        std::fs::write(trash.join("partial.bin"), b"partial").expect("partial trash bytes");
        let state = if rollback_cleanup {
            OllamaJournalState::RollbackCleanupPending {
                previous: active,
                rejected_target: Some(deleted_fingerprint),
            }
        } else {
            OllamaJournalState::CleanupPending {
                target: active,
                previous: deleted_fingerprint,
            }
        };
        write_real_journal(&paths, &OllamaTransactionJournal::new(state));
        let models_path = root.path().join("models");
        std::fs::create_dir_all(&models_path).expect("models directory");
        let models = NativePathIdentityResolver
            .canonical_directory(&models_path)
            .expect("models identity");
        let fs = Arc::new(platform_fs());
        let runner = RecoveryExecutor::new_with_models(
            Arc::clone(&fs),
            Arc::new(ValidProbe),
            paths.clone(),
            models,
        );

        assert_eq!(
            runner.recover(RecoveryReason::Startup).await,
            Ok(super::recovery::RecoveryOutcome::ProgressMade)
        );
        assert!(!trash.exists());
        assert!(paths.active.exists());
        assert!(paths.journal.exists());
        assert_eq!(
            runner.recover(RecoveryReason::Retry).await,
            Ok(super::recovery::RecoveryOutcome::ProgressMade)
        );
        assert!(!paths.journal.exists());
        assert_eq!(
            runner.recover(RecoveryReason::Retry).await,
            Ok(super::recovery::RecoveryOutcome::ProgressMade)
        );
        assert_eq!(
            runner.recover(RecoveryReason::Retry).await,
            Ok(super::recovery::RecoveryOutcome::Ready)
        );
    }
}

#[tokio::test]
async fn prepared_target_rejected_by_probe_rolls_back_to_the_previous_bundle() {
    let root = tempfile::tempdir_in(".").expect("layout root");
    let paths = ollama_paths(root.path());
    let target = real_bundle(root.path(), "ollama-bundle", "1.2.3", b"rejected");
    let previous = real_bundle(root.path(), "ollama-bundle-backup", "1.2.2", b"previous");
    write_real_journal(
        &paths,
        &OllamaTransactionJournal::new(OllamaJournalState::Prepared {
            target,
            previous: previous.clone(),
        }),
    );
    let models_path = root.path().join("models");
    std::fs::create_dir_all(&models_path).expect("models directory");
    let models = NativePathIdentityResolver
        .canonical_directory(&models_path)
        .expect("models identity");
    let fs = Arc::new(platform_fs());
    let runner = RecoveryExecutor::new_with_models(
        Arc::clone(&fs),
        Arc::new(InvalidProbe),
        paths.clone(),
        models,
    );

    for reason in [
        RecoveryReason::Startup,
        RecoveryReason::Retry,
        RecoveryReason::Retry,
        RecoveryReason::Retry,
        RecoveryReason::Retry,
        RecoveryReason::Retry,
        RecoveryReason::Retry,
    ] {
        assert_eq!(
            runner.recover(reason).await,
            Ok(super::recovery::RecoveryOutcome::ProgressMade)
        );
    }
    assert_eq!(
        runner.recover(RecoveryReason::Retry).await,
        Ok(super::recovery::RecoveryOutcome::ProgressMade)
    );
    assert_eq!(
        runner.recover(RecoveryReason::Retry).await,
        Ok(super::recovery::RecoveryOutcome::Ready)
    );
    assert_eq!(
        super::cleanup_inspection::fingerprint(&platform_fs(), &paths.active),
        Some(DirectoryEvidence::Present(previous))
    );
    assert!(!paths.backup.exists());
    assert!(!paths.failed.exists());
    assert!(!paths.failed_delete.exists());
    assert!(!paths.journal.exists());
}

#[cfg(unix)]
#[tokio::test]
async fn real_cleanup_rejects_an_alias_without_touching_the_target() {
    use std::os::unix::fs::symlink;

    let root = tempfile::tempdir_in(".").expect("layout root");
    let paths = ollama_paths(root.path());
    let active = real_bundle(root.path(), "ollama-bundle", "1.2.3", b"active");
    let outside = tempfile::tempdir_in(".").expect("outside");
    let outside_bundle = outside.path().join("outside-bundle");
    let _ = real_bundle(outside.path(), "outside-bundle", "1.2.2", b"outside");
    symlink(&outside_bundle, &paths.backup_delete).expect("alias");
    let journal = OllamaTransactionJournal::new(OllamaJournalState::CleanupPending {
        target: active,
        previous: fp("1.2.2", "22"),
    });
    std::fs::write(
        &paths.journal,
        serde_json::to_vec(&journal).expect("journal"),
    )
    .expect("journal");
    let models_path = root.path().join("models");
    std::fs::create_dir_all(&models_path).expect("models directory");
    let models = NativePathIdentityResolver
        .canonical_directory(&models_path)
        .expect("models identity");
    let fs = Arc::new(super::durable_fs::platform_fs());
    let runner = RecoveryExecutor::new_with_models(
        Arc::clone(&fs),
        Arc::new(ValidProbe),
        paths.clone(),
        models,
    );

    assert_eq!(
        runner.recover(RecoveryReason::Startup).await,
        Ok(super::recovery::RecoveryOutcome::Deferred {
            code: OllamaErrorCode::OllamaUpdateRecoveryRequired,
        })
    );
    assert!(outside_bundle.exists());
    assert!(paths.backup_delete.exists());
    assert!(paths.journal.exists());
}

#[tokio::test]
async fn real_recovery_restores_a_legacy_backup_before_adopting_it() {
    let root = tempfile::tempdir_in(".").expect("layout root");
    let paths = ollama_paths(root.path());
    let previous = real_bundle(root.path(), "ollama-bundle-old", "1.2.2", b"previous");
    let models_path = root.path().join("models");
    std::fs::create_dir_all(&models_path).expect("models directory");
    let models = NativePathIdentityResolver
        .canonical_directory(&models_path)
        .expect("models identity");
    let fs = Arc::new(super::durable_fs::platform_fs());
    let runner = RecoveryExecutor::new_with_models(
        Arc::clone(&fs),
        Arc::new(ValidProbe),
        paths.clone(),
        models,
    );

    assert_eq!(
        runner.recover(RecoveryReason::Startup).await,
        Ok(super::recovery::RecoveryOutcome::ProgressMade)
    );
    assert!(!paths.legacy_backup.exists());
    assert!(paths.active.exists());
    assert_eq!(
        cleanup::snapshot(JournalPresence::Absent, &*fs, &paths).legacy_backup,
        DirectoryEvidence::Absent
    );
    assert_eq!(
        runner.recover(RecoveryReason::Retry).await,
        Ok(super::recovery::RecoveryOutcome::ProgressMade)
    );
    assert!(paths.migration_marker.exists());
    assert_eq!(
        runner.recover(RecoveryReason::Retry).await,
        Ok(super::recovery::RecoveryOutcome::Ready)
    );
    let _ = previous;
}

#[tokio::test]
async fn real_recovery_rolls_back_the_rejected_bundle_and_cleans_its_rebut() {
    let root = tempfile::tempdir_in(".").expect("layout root");
    let paths = ollama_paths(root.path());
    let rejected = real_bundle(root.path(), "ollama-bundle", "1.2.3", b"rejected");
    let previous = real_bundle(root.path(), "ollama-bundle-backup", "1.2.2", b"previous");
    let journal = OllamaTransactionJournal::new(OllamaJournalState::RollbackPending {
        previous: previous.clone(),
        rejected_target: Some(rejected),
    });
    std::fs::write(
        &paths.journal,
        serde_json::to_vec(&journal).expect("journal"),
    )
    .expect("journal");
    let models_path = root.path().join("models");
    std::fs::create_dir_all(&models_path).expect("models directory");
    let models = NativePathIdentityResolver
        .canonical_directory(&models_path)
        .expect("models identity");
    let fs = Arc::new(super::durable_fs::platform_fs());
    let runner = RecoveryExecutor::new_with_models(
        Arc::clone(&fs),
        Arc::new(ValidProbe),
        paths.clone(),
        models,
    );

    for _ in 0..7 {
        let outcome = runner.recover(RecoveryReason::Retry).await;
        assert!(matches!(
            outcome,
            Ok(super::recovery::RecoveryOutcome::ProgressMade)
                | Ok(super::recovery::RecoveryOutcome::Ready)
        ));
    }
    assert!(paths.active.exists());
    assert!(!paths.failed.exists());
    assert!(!paths.failed_delete.exists());
    assert!(!paths.journal.exists());
}

#[tokio::test]
async fn real_recovery_keeps_source_and_rebut_when_the_layout_is_ambiguous() {
    let root = tempfile::tempdir_in(".").expect("layout root");
    let paths = ollama_paths(root.path());
    let active = real_bundle(root.path(), "ollama-bundle", "1.2.3", b"active");
    let previous = real_bundle(root.path(), "ollama-bundle-backup", "1.2.2", b"previous");
    let _ = real_bundle(
        root.path(),
        "ollama-bundle-backup-delete",
        "1.2.2",
        b"previous-delete",
    );
    let journal = OllamaTransactionJournal::new(OllamaJournalState::CleanupPending {
        target: active,
        previous,
    });
    std::fs::write(
        &paths.journal,
        serde_json::to_vec(&journal).expect("journal"),
    )
    .expect("journal");
    let models_path = root.path().join("models");
    std::fs::create_dir_all(&models_path).expect("models directory");
    let models = NativePathIdentityResolver
        .canonical_directory(&models_path)
        .expect("models identity");
    let fs = Arc::new(super::durable_fs::platform_fs());
    let runner = RecoveryExecutor::new_with_models(
        Arc::clone(&fs),
        Arc::new(ValidProbe),
        paths.clone(),
        models,
    );

    assert_eq!(
        runner.recover(RecoveryReason::Startup).await,
        Ok(super::recovery::RecoveryOutcome::Deferred {
            code: OllamaErrorCode::OllamaUpdateRecoveryRequired,
        })
    );
    assert!(paths.backup.exists());
    assert!(paths.backup_delete.exists());
    assert!(paths.journal.exists());
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
        let (_root, fs, _runner, paths) = setup();
        fs.seed(&paths.active);
        fs.fail_at(failure);
        assert_eq!(
            cleanup::write_marker(&fs, &paths).await,
            Err(OllamaErrorCode::OllamaStorageUnavailable)
        );
        fs.clear_failure();
        if fs.final_bytes.lock().unwrap().is_none() {
            cleanup::write_marker(&fs, &paths)
                .await
                .expect("marker retry");
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
