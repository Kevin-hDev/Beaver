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

pub(super) fn is_exactly_present(evidence: &DirectoryEvidence) -> bool {
    matches!(evidence, DirectoryEvidence::Present(_))
}

pub(super) fn is_present_or_incomplete(evidence: &DirectoryEvidence) -> bool {
    matches!(
        evidence,
        DirectoryEvidence::Present(_) | DirectoryEvidence::Incomplete
    )
}
