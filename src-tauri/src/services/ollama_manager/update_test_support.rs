use super::super::fingerprint::{BundleFingerprint, OllamaVersion};
use super::super::journal::{OllamaJournalState, OllamaTransactionJournal};
use super::super::path_identity::PathIdentityResolver;
use super::super::probe::{PreparedBundle, TargetValidation};
use super::super::recovery_decision::{
    decide_recovery, ArchiveDirectoryEvidence, DirectoryEvidence, JournalPresence,
    MigrationMarkerPresence, OllamaLayoutSnapshot, RecoveryDecision,
};
use super::super::update::{UpdateRequest, UpdateSidecar};
use std::path::Path;
use std::sync::Mutex;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum FailurePoint {
    Prepare,
    VersionBefore,
    VersionAfter,
    ReceiptBefore,
    ReceiptAfter,
    PreparedBefore,
    PreparedAfter,
    Stop,
    Reap,
    ActiveRenameBefore,
    ActiveRenameAfter,
    ActiveSyncBefore,
    ActiveSyncAfter,
    TargetRenameBefore,
    TargetRenameAfter,
    TargetSyncBefore,
    TargetSyncAfter,
    PendingBefore,
    PendingAfter,
    CleanupBefore,
    CleanupAfter,
    RollbackBefore,
    RollbackAfter,
}

pub(super) struct FakeBackend {
    pub(super) events: Mutex<Vec<&'static str>>,
    pub(super) metadata_events: Mutex<Vec<&'static str>>,
    pub(super) journal: Mutex<Option<OllamaTransactionJournal>>,
    pub(super) previous: BundleFingerprint,
    pub(super) target: Mutex<PreparedBundle>,
    pub(super) probe: Mutex<TargetValidation>,
    pub(super) failure: Mutex<Option<FailurePoint>>,
    pub(super) staging_authoritative: Mutex<bool>,
    pub(super) active_renamed: Mutex<bool>,
    pub(super) target_renamed: Mutex<bool>,
}

#[path = "update_test_backend.rs"]
mod backend;

impl FakeBackend {
    pub(super) fn new(root: &Path, target_version: &str) -> Self {
        let root = std::fs::canonicalize(root).expect("test root");
        let active = root.join("active");
        let target = root.join("update-staging");
        std::fs::create_dir_all(active.join("bin")).unwrap();
        std::fs::create_dir_all(target.join("bin")).unwrap();
        std::fs::copy("/usr/bin/true", active.join("bin/ollama")).unwrap();
        std::fs::copy("/usr/bin/false", target.join("bin/ollama")).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            for path in [active.join("bin/ollama"), target.join("bin/ollama")] {
                std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
            }
        }
        let resolver = super::super::path_identity_resolver::NativePathIdentityResolver;
        let active_executable = resolver
            .canonical_executable(&active.join("bin/ollama"))
            .unwrap();
        let target_root = resolver.canonical_directory(&target).unwrap();
        let target_executable = resolver
            .canonical_executable(&target.join("bin/ollama"))
            .unwrap();
        let previous = BundleFingerprint {
            version: OllamaVersion::parse("1.0.0").unwrap(),
            executable_sha256: super::super::probe_http::hash_file(active_executable.path())
                .ok()
                .expect("active digest"),
        };
        let fingerprint = BundleFingerprint {
            version: OllamaVersion::parse(target_version).unwrap(),
            executable_sha256: super::super::probe_http::hash_file(target_executable.path())
                .ok()
                .expect("target digest"),
        };
        Self {
            events: Mutex::new(Vec::new()),
            metadata_events: Mutex::new(Vec::new()),
            journal: Mutex::new(None),
            previous,
            target: Mutex::new(PreparedBundle {
                root: target_root,
                executable: target_executable,
                fingerprint: fingerprint.clone(),
            }),
            probe: Mutex::new(TargetValidation::Valid { fingerprint }),
            failure: Mutex::new(None),
            staging_authoritative: Mutex::new(false),
            active_renamed: Mutex::new(false),
            target_renamed: Mutex::new(false),
        }
    }

    pub(super) fn fail_at(&self, point: FailurePoint) {
        *self.failure.lock().unwrap() = Some(point);
    }

    pub(super) fn set_probe(&self, result: TargetValidation) {
        *self.probe.lock().unwrap() = result;
    }

    pub(super) fn make_current(&self) {
        let previous = self.previous.clone();
        self.target.lock().unwrap().fingerprint = previous;
    }

    pub(super) fn events(&self) -> Vec<&'static str> {
        self.events.lock().unwrap().clone()
    }

    pub(super) fn metadata_events(&self) -> Vec<&'static str> {
        self.metadata_events.lock().unwrap().clone()
    }

    pub(super) fn staging_authoritative(&self) -> bool {
        *self.staging_authoritative.lock().unwrap()
    }

    pub(super) fn journal_phase(&self) -> Option<OllamaJournalState> {
        self.journal
            .lock()
            .unwrap()
            .as_ref()
            .map(|j| j.state.clone())
    }

    pub(super) fn set_journal(&self, state: OllamaJournalState) {
        *self.journal.lock().unwrap() = Some(OllamaTransactionJournal::new(state));
    }

    pub(super) fn sidecar_is_owned(request: &UpdateRequest) -> bool {
        matches!(request.sidecar, UpdateSidecar::Owned(_))
    }

    pub(super) fn recovery_decision(&self) -> RecoveryDecision {
        let target = self.target.lock().unwrap().fingerprint.clone();
        let active_renamed = *self.active_renamed.lock().unwrap();
        let target_renamed = *self.target_renamed.lock().unwrap();
        let active = if target_renamed {
            DirectoryEvidence::Present(target)
        } else if active_renamed {
            DirectoryEvidence::Absent
        } else {
            DirectoryEvidence::Present(self.previous.clone())
        };
        let update_staging = if target_renamed {
            DirectoryEvidence::Absent
        } else {
            DirectoryEvidence::Present(self.target.lock().unwrap().fingerprint.clone())
        };
        let backup = if active_renamed {
            DirectoryEvidence::Present(self.previous.clone())
        } else {
            DirectoryEvidence::Absent
        };
        let journal = self
            .journal
            .lock()
            .unwrap()
            .clone()
            .map(JournalPresence::Valid)
            .unwrap_or(JournalPresence::Absent);
        decide_recovery(&OllamaLayoutSnapshot {
            journal,
            migration_marker: MigrationMarkerPresence::Absent,
            active,
            install_staging: DirectoryEvidence::Absent,
            archive_staging: ArchiveDirectoryEvidence::Absent,
            archive_failed: ArchiveDirectoryEvidence::Absent,
            update_staging,
            backup,
            failed: DirectoryEvidence::Absent,
            legacy_staging: DirectoryEvidence::Absent,
            legacy_backup: DirectoryEvidence::Absent,
            backup_delete: DirectoryEvidence::Absent,
            failed_delete: DirectoryEvidence::Absent,
        })
    }
}
