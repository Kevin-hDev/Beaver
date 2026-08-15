use super::super::error::OllamaErrorCode;
use super::super::fingerprint::BundleFingerprint;
use super::super::journal::{OllamaJournalState, OllamaTransactionJournal};
use super::{UpdateBackend, UpdateOutcome};

const MAX_COMPLETION_PASSES: usize = 8;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ValidatedJournal {
    target: BundleFingerprint,
    previous: BundleFingerprint,
}

impl ValidatedJournal {
    pub(crate) fn from_pending(
        journal: &OllamaTransactionJournal,
        observed: &BundleFingerprint,
    ) -> Result<Self, OllamaErrorCode> {
        match &journal.state {
            OllamaJournalState::PendingValidation { target, previous }
                if target == observed && target != previous =>
            {
                Ok(Self {
                    target: target.clone(),
                    previous: previous.clone(),
                })
            }
            _ => Err(OllamaErrorCode::OllamaUpdateRecoveryRequired),
        }
    }

    #[cfg(test)]
    pub(crate) fn target(&self) -> &BundleFingerprint {
        &self.target
    }

    fn cleanup_state(&self) -> OllamaJournalState {
        OllamaJournalState::CleanupPending {
            target: self.target.clone(),
            previous: self.previous.clone(),
        }
    }

    fn matches_pending(&self, journal: &OllamaTransactionJournal) -> bool {
        matches!(
            &journal.state,
            OllamaJournalState::PendingValidation { target, previous }
                if target == &self.target && previous == &self.previous
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RejectedJournal {
    previous: BundleFingerprint,
    rejected_target: BundleFingerprint,
    code: OllamaErrorCode,
}

impl RejectedJournal {
    pub(crate) fn from_pending(
        journal: &OllamaTransactionJournal,
        rejected_target: &BundleFingerprint,
        code: OllamaErrorCode,
    ) -> Result<Self, OllamaErrorCode> {
        match &journal.state {
            OllamaJournalState::PendingValidation { target, previous }
                if target == rejected_target && target != previous =>
            {
                Ok(Self {
                    previous: previous.clone(),
                    rejected_target: rejected_target.clone(),
                    code,
                })
            }
            _ => Err(OllamaErrorCode::OllamaUpdateRecoveryRequired),
        }
    }

    #[cfg(test)]
    pub(crate) fn rejected_target(&self) -> &BundleFingerprint {
        &self.rejected_target
    }

    fn rollback_state(&self) -> OllamaJournalState {
        OllamaJournalState::RollbackPending {
            previous: self.previous.clone(),
            rejected_target: Some(self.rejected_target.clone()),
        }
    }

    fn matches_pending(&self, journal: &OllamaTransactionJournal) -> bool {
        matches!(
            &journal.state,
            OllamaJournalState::PendingValidation { target, previous }
                if target == &self.rejected_target && previous == &self.previous
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CompletionRecovery {
    Ready,
    Progress,
    Deferred { code: OllamaErrorCode },
}

pub(crate) async fn complete_valid_update<B: UpdateBackend>(
    backend: &B,
    journal: ValidatedJournal,
) -> Result<UpdateOutcome, OllamaErrorCode> {
    ensure_pending(backend, |current| journal.matches_pending(current)).await?;
    backend.persist(journal.cleanup_state(), true).await?;
    for _ in 0..MAX_COMPLETION_PASSES {
        match backend.recover_completion().await {
            Ok(CompletionRecovery::Ready) => {
                return Ok(UpdateOutcome::Updated {
                    fingerprint: journal.target,
                });
            }
            Ok(CompletionRecovery::Progress) => continue,
            Ok(CompletionRecovery::Deferred { code }) => {
                return Ok(UpdateOutcome::CleanupPending { code });
            }
            Err(_) => {
                return Ok(UpdateOutcome::CleanupPending {
                    code: OllamaErrorCode::OllamaUpdateCleanupPending,
                });
            }
        }
    }
    Ok(UpdateOutcome::CleanupPending {
        code: OllamaErrorCode::OllamaUpdateCleanupPending,
    })
}

pub(crate) async fn reject_target_and_restore<B: UpdateBackend>(
    backend: &B,
    journal: RejectedJournal,
) -> Result<UpdateOutcome, OllamaErrorCode> {
    ensure_pending(backend, |current| journal.matches_pending(current)).await?;
    backend.persist(journal.rollback_state(), true).await?;
    for _ in 0..MAX_COMPLETION_PASSES {
        match backend.recover_completion().await {
            Ok(CompletionRecovery::Ready) => {
                return Ok(UpdateOutcome::Deferred { code: journal.code });
            }
            Ok(CompletionRecovery::Progress) => continue,
            Ok(CompletionRecovery::Deferred { .. }) | Err(_) => {
                return Ok(UpdateOutcome::Deferred { code: journal.code });
            }
        }
    }
    Ok(UpdateOutcome::Deferred { code: journal.code })
}

async fn ensure_pending<B, F>(backend: &B, matches: F) -> Result<(), OllamaErrorCode>
where
    B: UpdateBackend,
    F: Fn(&OllamaTransactionJournal) -> bool,
{
    match backend.journal().await? {
        Some(journal) if matches(&journal) => Ok(()),
        _ => Err(OllamaErrorCode::OllamaUpdateRecoveryRequired),
    }
}
