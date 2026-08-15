use super::fingerprint::BundleFingerprint;
use super::journal::{OllamaJournalState, OllamaTransactionJournal};
use super::recovery_decision::DirectoryEvidence;

pub(super) fn target_of(journal: &OllamaTransactionJournal) -> Option<&BundleFingerprint> {
    match &journal.state {
        OllamaJournalState::Prepared { target, .. }
        | OllamaJournalState::PendingValidation { target, .. }
        | OllamaJournalState::CleanupPending { target, .. } => Some(target),
        _ => None,
    }
}

pub(super) fn present(evidence: &DirectoryEvidence) -> bool {
    matches!(evidence, DirectoryEvidence::Present(_))
}
