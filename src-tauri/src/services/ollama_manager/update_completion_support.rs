use super::error::OllamaErrorCode;
use super::fingerprint::{BundleFingerprint, OllamaVersion, Sha256Digest};
use super::journal::{OllamaJournalState, OllamaTransactionJournal};
use super::probe::{PreparedBundle, TargetValidation};
use super::update::{CompletionRecovery, RejectedJournal, UpdateBackend, UpdateRequest};
use async_trait::async_trait;
use std::sync::Mutex;

pub(crate) fn fingerprint(version: &str, byte: &str) -> BundleFingerprint {
    BundleFingerprint {
        version: OllamaVersion::parse(version).unwrap(),
        executable_sha256: Sha256Digest::from_hex(&byte.repeat(32)).unwrap(),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CompletionCutpoint {
    BackupMoveBefore,
    BackupMoveAfter,
    BackupDeleteBefore,
    BackupDeleteAfter,
    FailedMoveBefore,
    FailedMoveAfter,
    RestoreBefore,
    RestoreAfter,
    RollbackJournalBefore,
    RollbackJournalAfter,
    FailedDeleteMoveBefore,
    FailedDeleteMoveAfter,
    FailedDeleteBefore,
    FailedDeleteAfter,
    JournalRemoveBefore,
    JournalRemoveAfter,
}

#[derive(Clone, Debug)]
pub(crate) struct Layout {
    pub(crate) active: Option<BundleFingerprint>,
    pub(crate) backup: Option<BundleFingerprint>,
    pub(crate) failed: Option<BundleFingerprint>,
    pub(crate) backup_delete: Option<BundleFingerprint>,
    pub(crate) failed_delete: Option<BundleFingerprint>,
    pub(crate) journal: Option<OllamaTransactionJournal>,
    pub(crate) models: Vec<u8>,
}

pub(crate) struct CompletionHarness {
    pub(crate) previous: BundleFingerprint,
    pub(crate) target: BundleFingerprint,
    pub(crate) state: Mutex<Layout>,
    failure: Mutex<Option<CompletionCutpoint>>,
}

impl CompletionHarness {
    pub(crate) fn valid() -> Self {
        let previous = fingerprint("1.0.0", "11");
        let target = fingerprint("2.0.0", "22");
        Self {
            previous: previous.clone(),
            target: target.clone(),
            state: Mutex::new(Layout {
                active: Some(target),
                backup: Some(previous),
                failed: None,
                backup_delete: None,
                failed_delete: None,
                journal: None,
                models: b"model-store".to_vec(),
            }),
            failure: Mutex::new(None),
        }
    }

    pub(crate) fn pending(&self) -> OllamaTransactionJournal {
        OllamaTransactionJournal::new(OllamaJournalState::PendingValidation {
            target: self.target.clone(),
            previous: self.previous.clone(),
        })
    }

    pub(crate) fn rejected(&self) -> RejectedJournal {
        RejectedJournal::from_pending(
            &self.pending(),
            &self.target,
            OllamaErrorCode::OllamaBundleInvalid,
        )
        .unwrap()
    }

    pub(crate) fn set_pending(&self) {
        self.state.lock().unwrap().journal = Some(self.pending());
    }

    pub(crate) fn fail_once(&self, point: CompletionCutpoint) {
        *self.failure.lock().unwrap() = Some(point);
    }

    pub(crate) fn clear_failure(&self) {
        *self.failure.lock().unwrap() = None;
    }

    pub(crate) fn journal_state(&self) -> Option<OllamaJournalState> {
        self.state
            .lock()
            .unwrap()
            .journal
            .as_ref()
            .map(|j| j.state.clone())
    }

    pub(crate) fn active(&self) -> Option<BundleFingerprint> {
        self.state.lock().unwrap().active.clone()
    }

    pub(crate) fn backup(&self) -> Option<BundleFingerprint> {
        self.state.lock().unwrap().backup.clone()
    }

    pub(crate) fn failed(&self) -> Option<BundleFingerprint> {
        self.state.lock().unwrap().failed.clone()
    }

    pub(crate) fn models(&self) -> Vec<u8> {
        self.state.lock().unwrap().models.clone()
    }

    pub(crate) fn assert_safe_layout(&self) {
        let state = self.state.lock().unwrap();
        assert!(state.active.is_some() || state.backup.is_some() || state.failed.is_some());
        assert!(!(state.active.is_none() && state.backup.is_none() && state.failed.is_none()));
    }

    pub(crate) async fn drain(&self) {
        for _ in 0..8 {
            if self.journal_state().is_none() {
                return;
            }
            let result = <Self as UpdateBackend>::recover_completion(self).await;
            assert!(result.is_ok(), "recovery did not converge: {result:?}");
            self.assert_safe_layout();
        }
        panic!("completion did not converge");
    }

    pub(crate) fn fail_at(&self, point: CompletionCutpoint) -> Result<(), OllamaErrorCode> {
        let mut failure = self.failure.lock().unwrap();
        if *failure == Some(point) {
            *failure = None;
            return Err(OllamaErrorCode::OllamaStorageUnavailable);
        }
        Ok(())
    }

    pub(crate) fn persist_state(&self, state: OllamaJournalState) {
        self.state.lock().unwrap().journal = Some(OllamaTransactionJournal::new(state));
    }
}

#[async_trait]
impl UpdateBackend for CompletionHarness {
    async fn journal(&self) -> Result<Option<OllamaTransactionJournal>, OllamaErrorCode> {
        Ok(self.state.lock().unwrap().journal.clone())
    }

    async fn current(&self) -> Result<BundleFingerprint, OllamaErrorCode> {
        Ok(self.previous.clone())
    }

    async fn prepare_target(
        &self,
        _request: &UpdateRequest,
    ) -> Result<PreparedBundle, OllamaErrorCode> {
        Err(OllamaErrorCode::OllamaInternal)
    }

    async fn persist(
        &self,
        state: OllamaJournalState,
        _replace: bool,
    ) -> Result<(), OllamaErrorCode> {
        self.persist_state(state);
        Ok(())
    }

    async fn stop_owned_sidecar(&self, _request: &UpdateRequest) -> Result<(), OllamaErrorCode> {
        Ok(())
    }

    async fn reap_owned_sidecar(&self, _request: &UpdateRequest) -> Result<(), OllamaErrorCode> {
        Ok(())
    }

    async fn rename_active_to_backup(&self) -> Result<(), OllamaErrorCode> {
        Ok(())
    }

    async fn rename_target_to_active(&self) -> Result<(), OllamaErrorCode> {
        Ok(())
    }

    async fn probe_active(
        &self,
        _request: &UpdateRequest,
        _target: &BundleFingerprint,
    ) -> TargetValidation {
        TargetValidation::Deferred {
            code: OllamaErrorCode::OllamaInternal,
        }
    }

    async fn recover_completion(&self) -> Result<CompletionRecovery, OllamaErrorCode> {
        self.recover_once()
    }
}

#[path = "update_completion_support_state.rs"]
mod state;
