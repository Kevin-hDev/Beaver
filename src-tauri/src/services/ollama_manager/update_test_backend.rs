use super::super::super::error::OllamaErrorCode;
use super::super::super::fingerprint::BundleFingerprint;
use super::super::super::journal::{OllamaJournalState, OllamaTransactionJournal};
use super::super::super::probe::{PreparedBundle, TargetValidation};
use super::super::super::update::{UpdateBackend, UpdateRequest};
use super::{FailurePoint, FakeBackend};
use async_trait::async_trait;

impl FakeBackend {
    fn fail(&self, point: FailurePoint) -> bool {
        *self.failure.lock().unwrap() == Some(point)
    }

    fn event(&self, value: &'static str) {
        self.events.lock().unwrap().push(value);
    }
}

#[async_trait]
impl UpdateBackend for FakeBackend {
    async fn journal(&self) -> Result<Option<OllamaTransactionJournal>, OllamaErrorCode> {
        Ok(self.journal.lock().unwrap().clone())
    }

    async fn current(
        &self,
    ) -> Result<super::super::super::fingerprint::BundleFingerprint, OllamaErrorCode> {
        Ok(self.previous.clone())
    }

    async fn prepare_target(
        &self,
        _request: &UpdateRequest,
    ) -> Result<PreparedBundle, OllamaErrorCode> {
        self.event("prepare_target");
        if self.fail(FailurePoint::Prepare) {
            return Err(OllamaErrorCode::OllamaBundleInvalid);
        }
        if self.fail(FailurePoint::VersionBefore) {
            return Err(OllamaErrorCode::OllamaStorageUnavailable);
        }
        self.metadata_events.lock().unwrap().push("VERSION");
        if self.fail(FailurePoint::VersionAfter) {
            return Err(OllamaErrorCode::OllamaStorageUnavailable);
        }
        if self.fail(FailurePoint::ReceiptBefore) {
            return Err(OllamaErrorCode::OllamaStorageUnavailable);
        }
        self.metadata_events.lock().unwrap().push("receipt");
        if self.fail(FailurePoint::ReceiptAfter) {
            return Err(OllamaErrorCode::OllamaStorageUnavailable);
        }
        *self.staging_authoritative.lock().unwrap() = true;
        Ok(self.target.lock().unwrap().clone())
    }

    async fn persist(
        &self,
        state: OllamaJournalState,
        replace: bool,
    ) -> Result<(), OllamaErrorCode> {
        let (before, after, name) = match state {
            OllamaJournalState::Prepared { .. } => (
                FailurePoint::PreparedBefore,
                FailurePoint::PreparedAfter,
                "persist_prepared",
            ),
            OllamaJournalState::PendingValidation { .. } => (
                FailurePoint::PendingBefore,
                FailurePoint::PendingAfter,
                "persist_pending_validation",
            ),
            OllamaJournalState::CleanupPending { .. } => (
                FailurePoint::CleanupBefore,
                FailurePoint::CleanupAfter,
                "persist_cleanup_pending",
            ),
            OllamaJournalState::RollbackPending { .. } => (
                FailurePoint::RollbackBefore,
                FailurePoint::RollbackAfter,
                "persist_rollback_pending",
            ),
            OllamaJournalState::RollbackCleanupPending { .. } => unreachable!(),
        };
        self.event(name);
        if self.fail(before) {
            return Err(OllamaErrorCode::OllamaStorageUnavailable);
        }
        let value = OllamaTransactionJournal::new(state);
        if replace || self.journal.lock().unwrap().is_none() {
            *self.journal.lock().unwrap() = Some(value);
        } else {
            return Err(OllamaErrorCode::OllamaStorageUnavailable);
        }
        if self.fail(after) {
            return Err(OllamaErrorCode::OllamaStorageUnavailable);
        }
        Ok(())
    }

    async fn stop_owned_sidecar(&self, request: &UpdateRequest) -> Result<(), OllamaErrorCode> {
        if !FakeBackend::sidecar_is_owned(request) {
            return Ok(());
        }
        self.event("stop_owned_sidecar");
        (!self.fail(FailurePoint::Stop))
            .then_some(())
            .ok_or(OllamaErrorCode::OllamaStopFailed)
    }

    async fn reap_owned_sidecar(&self, request: &UpdateRequest) -> Result<(), OllamaErrorCode> {
        if !FakeBackend::sidecar_is_owned(request) {
            return Ok(());
        }
        self.event("reap_owned_sidecar");
        (!self.fail(FailurePoint::Reap))
            .then_some(())
            .ok_or(OllamaErrorCode::OllamaStopFailed)
    }

    async fn rename_active_to_backup(&self) -> Result<(), OllamaErrorCode> {
        self.event("rename_active_to_backup");
        if self.fail(FailurePoint::ActiveRenameBefore) {
            return Err(OllamaErrorCode::OllamaStorageUnavailable);
        }
        *self.active_renamed.lock().unwrap() = true;
        if self.fail(FailurePoint::ActiveRenameAfter) {
            return Err(OllamaErrorCode::OllamaStorageUnavailable);
        }
        if self.fail(FailurePoint::ActiveSyncBefore) {
            return Err(OllamaErrorCode::OllamaStorageUnavailable);
        }
        self.event("sync_parent_active_backup");
        (!self.fail(FailurePoint::ActiveSyncAfter))
            .then_some(())
            .ok_or(OllamaErrorCode::OllamaStorageUnavailable)
    }

    async fn rename_target_to_active(&self) -> Result<(), OllamaErrorCode> {
        self.event("rename_target_to_active");
        if self.fail(FailurePoint::TargetRenameBefore) {
            return Err(OllamaErrorCode::OllamaStorageUnavailable);
        }
        *self.target_renamed.lock().unwrap() = true;
        if self.fail(FailurePoint::TargetRenameAfter) {
            return Err(OllamaErrorCode::OllamaStorageUnavailable);
        }
        if self.fail(FailurePoint::TargetSyncBefore) {
            return Err(OllamaErrorCode::OllamaStorageUnavailable);
        }
        self.event("sync_parent_target_active");
        (!self.fail(FailurePoint::TargetSyncAfter))
            .then_some(())
            .ok_or(OllamaErrorCode::OllamaStorageUnavailable)
    }

    async fn probe_active(
        &self,
        _request: &UpdateRequest,
        _target: &BundleFingerprint,
    ) -> TargetValidation {
        self.event("probe_active");
        self.probe.lock().unwrap().clone()
    }
}
