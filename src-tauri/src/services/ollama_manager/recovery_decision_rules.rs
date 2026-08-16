#![allow(dead_code)]

use super::journal::{OllamaJournalState, OllamaTransactionJournal};
use super::migration::BackupPolicy;
use super::recovery_decision::{
    exact_mask, mask, recovery_required, OllamaLayoutSnapshot, RecoveryDecision,
};

fn without_journal(s: &OllamaLayoutSnapshot, policy: BackupPolicy) -> RecoveryDecision {
    let m = mask(s);
    match m {
        0 => RecoveryDecision::Ready,
        1 => match s.migration_marker {
            super::recovery_decision::MigrationMarkerPresence::Absent
            | super::recovery_decision::MigrationMarkerPresence::Temporary => {
                RecoveryDecision::AdoptLegacyActive
            }
            super::recovery_decision::MigrationMarkerPresence::Valid(_) => RecoveryDecision::Ready,
            super::recovery_decision::MigrationMarkerPresence::Invalid
            | super::recovery_decision::MigrationMarkerPresence::Unknown => recovery_required(),
        },
        8 if matches!(
            policy,
            BackupPolicy::LegacyAndModern | BackupPolicy::ModernOnly
        ) =>
        {
            RecoveryDecision::RestoreLegacyBackup
        }
        64 if matches!(policy, BackupPolicy::LegacyAndModern) => {
            RecoveryDecision::RestoreLegacyBackup
        }
        _ => recovery_required(),
    }
}

fn with_journal(s: &OllamaLayoutSnapshot, journal: &OllamaTransactionJournal) -> RecoveryDecision {
    let m = mask(s);
    if m & (2 | 32 | 64) != 0 {
        return recovery_required();
    }
    match &journal.state {
        OllamaJournalState::Prepared { target, previous } => match m {
            5 if exact_mask(s, 5, &[(1, previous), (4, target)]) => {
                RecoveryDecision::CommitFreshInstall
            }
            12 if exact_mask(s, 12, &[(4, target), (8, previous)]) => {
                RecoveryDecision::CommitFreshInstall
            }
            9 if exact_mask(s, 9, &[(1, target), (8, previous)]) => {
                RecoveryDecision::ResumeTargetValidation
            }
            1 if exact_mask(s, 1, &[(1, previous)]) => {
                RecoveryDecision::RemoveCompletedLegacyJournal
            }
            8 if exact_mask(s, 8, &[(8, previous)]) => RecoveryDecision::ResumeRollback,
            _ => recovery_required(),
        },
        OllamaJournalState::PendingValidation { target, previous } => match m {
            9 if exact_mask(s, 9, &[(1, target), (8, previous)]) => {
                RecoveryDecision::ResumeTargetValidation
            }
            8 if exact_mask(s, 8, &[(8, previous)]) => RecoveryDecision::ResumeRollback,
            _ => recovery_required(),
        },
        OllamaJournalState::CleanupPending { target, previous } => match m {
            9 if exact_mask(s, 9, &[(1, target), (8, previous)]) => RecoveryDecision::ResumeCleanup,
            129 if exact_mask(s, 129, &[(1, target)]) => RecoveryDecision::ResumeCleanup,
            1 if exact_mask(s, 1, &[(1, target)]) => RecoveryDecision::RemoveCompletedLegacyJournal,
            8 if exact_mask(s, 8, &[(8, previous)]) => RecoveryDecision::ResumeRollback,
            _ => recovery_required(),
        },
        OllamaJournalState::RollbackPending {
            previous,
            rejected_target,
        } => match rejected_target {
            None => match m {
                1 if exact_mask(s, 1, &[(1, previous)]) => {
                    RecoveryDecision::RemoveCompletedLegacyJournal
                }
                8 if exact_mask(s, 8, &[(8, previous)]) => RecoveryDecision::ResumeRollback,
                _ => recovery_required(),
            },
            Some(rejected) => match m {
                9 if exact_mask(s, 9, &[(1, rejected), (8, previous)]) => {
                    RecoveryDecision::ResumeRollback
                }
                24 if exact_mask(s, 24, &[(8, previous), (16, rejected)]) => {
                    RecoveryDecision::ResumeRollback
                }
                17 if exact_mask(s, 17, &[(1, previous), (16, rejected)]) => {
                    RecoveryDecision::ResumeRollbackCleanup
                }
                _ => recovery_required(),
            },
        },
        OllamaJournalState::RollbackCleanupPending {
            previous,
            rejected_target,
        } => match rejected_target {
            None => {
                if exact_mask(s, 1, &[(1, previous)]) {
                    RecoveryDecision::RemoveCompletedLegacyJournal
                } else {
                    recovery_required()
                }
            }
            Some(rejected) => match m {
                17 if exact_mask(s, 17, &[(1, previous), (16, rejected)]) => {
                    RecoveryDecision::ResumeRollbackCleanup
                }
                257 if exact_mask(s, 257, &[(1, previous)]) => {
                    RecoveryDecision::ResumeRollbackCleanup
                }
                1 if exact_mask(s, 1, &[(1, previous)]) => {
                    RecoveryDecision::RemoveCompletedLegacyJournal
                }
                _ => recovery_required(),
            },
        },
    }
}

pub(super) fn decide_without_journal(
    s: &OllamaLayoutSnapshot,
    policy: BackupPolicy,
) -> RecoveryDecision {
    without_journal(s, policy)
}

pub(super) fn decide_with_journal(
    s: &OllamaLayoutSnapshot,
    journal: &OllamaTransactionJournal,
) -> RecoveryDecision {
    with_journal(s, journal)
}
