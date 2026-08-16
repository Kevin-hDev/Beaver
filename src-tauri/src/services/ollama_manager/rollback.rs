#![allow(dead_code)]

use super::cleanup;
use super::durable_fs::OllamaDurableFs;
use super::error::OllamaErrorCode;
use super::journal::{OllamaJournalState, OllamaTransactionJournal};
use super::journal_store::OllamaJournalStore;
use super::path_identity::CanonicalDirectory;
use super::recovery_decision::{DirectoryEvidence, JournalPresence, OllamaLayoutSnapshot};
use crate::services::paths::OllamaPaths;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RollbackTransition {
    PersistRollbackPending,
    MoveRejectedToFailed,
    RestorePrevious,
    PersistRollbackCleanupPending,
    MoveFailedToDelete,
    RemoveFailedDelete,
    RemoveJournal,
}

pub(crate) fn choose(
    snapshot: &OllamaLayoutSnapshot,
) -> Result<RollbackTransition, OllamaErrorCode> {
    let active = present(&snapshot.active);
    let backup = present(&snapshot.backup);
    let failed = present(&snapshot.failed);
    let failed_delete = present(&snapshot.failed_delete);
    if failed && failed_delete {
        return Err(OllamaErrorCode::OllamaRecoveryDeferred);
    }
    let cleanup_phase = matches!(snapshot.journal, JournalPresence::Valid(ref journal) if matches!(journal.state, OllamaJournalState::RollbackCleanupPending { .. }));
    if cleanup_phase {
        if active && failed {
            return Ok(RollbackTransition::MoveFailedToDelete);
        }
        if active && failed_delete {
            return Ok(RollbackTransition::RemoveFailedDelete);
        }
        return Ok(RollbackTransition::RemoveJournal);
    }
    if active && !failed {
        return Ok(RollbackTransition::MoveRejectedToFailed);
    }
    if !active && backup {
        return Ok(RollbackTransition::RestorePrevious);
    }
    if active && failed {
        return Ok(RollbackTransition::PersistRollbackCleanupPending);
    }
    Ok(RollbackTransition::RemoveJournal)
}

pub(crate) fn rejected_state(journal: &OllamaTransactionJournal) -> Option<OllamaJournalState> {
    match &journal.state {
        OllamaJournalState::Prepared { target, previous }
        | OllamaJournalState::PendingValidation { target, previous } => {
            Some(OllamaJournalState::RollbackPending {
                previous: previous.clone(),
                rejected_target: Some(target.clone()),
            })
        }
        _ => None,
    }
}

pub(crate) fn cleanup_state(journal: &OllamaTransactionJournal) -> Option<OllamaJournalState> {
    match &journal.state {
        OllamaJournalState::RollbackPending {
            previous,
            rejected_target,
        } => Some(OllamaJournalState::RollbackCleanupPending {
            previous: previous.clone(),
            rejected_target: rejected_target.clone(),
        }),
        _ => None,
    }
}

pub(crate) async fn apply<F>(
    transition: RollbackTransition,
    snapshot: &OllamaLayoutSnapshot,
    fs: &Arc<F>,
    journal: &OllamaJournalStore<F>,
    paths: &OllamaPaths,
    models: Option<&CanonicalDirectory>,
) -> Result<(), OllamaErrorCode>
where
    F: OllamaDurableFs + 'static,
{
    match transition {
        RollbackTransition::MoveRejectedToFailed => {
            cleanup::rename(fs, &paths.active, &paths.failed).await
        }
        RollbackTransition::RestorePrevious => {
            cleanup::rename(fs, &paths.backup, &paths.active).await
        }
        RollbackTransition::PersistRollbackCleanupPending => {
            let current = match &snapshot.journal {
                JournalPresence::Valid(value) => value,
                _ => return Err(OllamaErrorCode::OllamaJournalInvalid),
            };
            let state = cleanup_state(current).ok_or(OllamaErrorCode::OllamaJournalInvalid)?;
            journal.replace(&OllamaTransactionJournal::new(state)).await
        }
        RollbackTransition::MoveFailedToDelete => {
            cleanup::rename(fs, &paths.failed, &paths.failed_delete).await
        }
        RollbackTransition::RemoveFailedDelete => {
            cleanup::remove_trash(fs, &paths.failed_delete, paths, models).await
        }
        RollbackTransition::RemoveJournal => journal.remove().await,
        RollbackTransition::PersistRollbackPending => Ok(()),
    }
}

fn present(evidence: &DirectoryEvidence) -> bool {
    matches!(
        evidence,
        DirectoryEvidence::Present(_) | DirectoryEvidence::Incomplete
    )
}
