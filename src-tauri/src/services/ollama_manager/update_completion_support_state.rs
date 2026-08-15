use super::{CompletionCutpoint, CompletionHarness, CompletionRecovery};
use crate::services::ollama_manager::error::OllamaErrorCode;
use crate::services::ollama_manager::journal::{OllamaJournalState, OllamaTransactionJournal};

impl CompletionHarness {
    pub(crate) fn recover_once(&self) -> Result<CompletionRecovery, OllamaErrorCode> {
        let mut state = self.state.lock().unwrap();
        let Some(journal) = state.journal.as_ref().map(|j| j.state.clone()) else {
            return Ok(CompletionRecovery::Ready);
        };
        match journal {
            OllamaJournalState::CleanupPending { .. } => self.recover_valid(&mut state),
            OllamaJournalState::RollbackPending {
                previous,
                rejected_target: Some(rejected),
            } => self.recover_rollback_pending(&mut state, previous, rejected),
            OllamaJournalState::RollbackCleanupPending {
                previous,
                rejected_target: Some(rejected),
            } => self.recover_rollback_cleanup(&mut state, previous, rejected),
            _ => Ok(CompletionRecovery::Deferred {
                code: OllamaErrorCode::OllamaUpdateRecoveryRequired,
            }),
        }
    }

    fn recover_valid(
        &self,
        state: &mut super::Layout,
    ) -> Result<CompletionRecovery, OllamaErrorCode> {
        if let Some(backup) = state.backup.take() {
            self.fail_at(CompletionCutpoint::BackupMoveBefore)?;
            state.backup_delete = Some(backup);
            return self.after_move(CompletionCutpoint::BackupMoveAfter);
        }
        if state.backup_delete.is_some() {
            self.fail_at(CompletionCutpoint::BackupDeleteBefore)?;
            state.backup_delete = None;
            return self.after_move(CompletionCutpoint::BackupDeleteAfter);
        }
        self.remove_journal(state)
    }

    fn recover_rollback_pending(
        &self,
        state: &mut super::Layout,
        previous: super::BundleFingerprint,
        rejected: super::BundleFingerprint,
    ) -> Result<CompletionRecovery, OllamaErrorCode> {
        if state.active.as_ref() == Some(&rejected) && state.failed.is_none() {
            self.fail_at(CompletionCutpoint::FailedMoveBefore)?;
            state.failed = state.active.take();
            return self.after_move(CompletionCutpoint::FailedMoveAfter);
        }
        if state.active.is_none() && state.backup.as_ref() == Some(&previous) {
            self.fail_at(CompletionCutpoint::RestoreBefore)?;
            state.active = state.backup.take();
            return self.after_move(CompletionCutpoint::RestoreAfter);
        }
        if state.active.as_ref() == Some(&previous) && state.failed.as_ref() == Some(&rejected) {
            self.fail_at(CompletionCutpoint::RollbackJournalBefore)?;
            state.journal = Some(OllamaTransactionJournal::new(
                OllamaJournalState::RollbackCleanupPending {
                    previous,
                    rejected_target: Some(rejected),
                },
            ));
            return self.after_move(CompletionCutpoint::RollbackJournalAfter);
        }
        Ok(CompletionRecovery::Deferred {
            code: OllamaErrorCode::OllamaRecoveryDeferred,
        })
    }

    fn recover_rollback_cleanup(
        &self,
        state: &mut super::Layout,
        previous: super::BundleFingerprint,
        rejected: super::BundleFingerprint,
    ) -> Result<CompletionRecovery, OllamaErrorCode> {
        if state.active.as_ref() != Some(&previous) {
            return Ok(CompletionRecovery::Deferred {
                code: OllamaErrorCode::OllamaRecoveryDeferred,
            });
        }
        if state.failed.as_ref() == Some(&rejected) {
            self.fail_at(CompletionCutpoint::FailedDeleteMoveBefore)?;
            state.failed_delete = state.failed.take();
            return self.after_move(CompletionCutpoint::FailedDeleteMoveAfter);
        }
        if state.failed_delete.as_ref() == Some(&rejected) {
            self.fail_at(CompletionCutpoint::FailedDeleteBefore)?;
            state.failed_delete = None;
            return self.after_move(CompletionCutpoint::FailedDeleteAfter);
        }
        self.remove_journal(state)
    }

    fn after_move(&self, point: CompletionCutpoint) -> Result<CompletionRecovery, OllamaErrorCode> {
        if self.fail_at(point).is_err() {
            return Err(OllamaErrorCode::OllamaStorageUnavailable);
        }
        Ok(CompletionRecovery::Progress)
    }

    fn remove_journal(
        &self,
        state: &mut super::Layout,
    ) -> Result<CompletionRecovery, OllamaErrorCode> {
        self.fail_at(CompletionCutpoint::JournalRemoveBefore)?;
        state.journal = None;
        if self.fail_at(CompletionCutpoint::JournalRemoveAfter).is_err() {
            return Err(OllamaErrorCode::OllamaStorageUnavailable);
        }
        Ok(CompletionRecovery::Ready)
    }
}
