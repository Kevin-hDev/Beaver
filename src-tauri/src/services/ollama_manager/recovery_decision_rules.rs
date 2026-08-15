#![allow(dead_code)]

use super::journal::{OllamaJournalState, OllamaTransactionJournal};
use super::migration::BackupPolicy;
use super::recovery_decision::{defer, exact_mask, mask, OllamaLayoutSnapshot, RecoveryDecision};

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
            | super::recovery_decision::MigrationMarkerPresence::Unknown => defer(),
        },
        2 => RecoveryDecision::RemoveUncommittedInstallStaging,
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
        5 => RecoveryDecision::RemoveUncommittedInstallStaging,
        33 if matches!(policy, BackupPolicy::LegacyAndModern) => {
            RecoveryDecision::RemoveUncommittedInstallStaging
        }
        65 if matches!(policy, BackupPolicy::LegacyAndModern) => {
            RecoveryDecision::ResumeTargetValidation
        }
        17 => RecoveryDecision::ResumeRollbackCleanup,
        _ => defer(),
    }
}

fn with_journal(s: &OllamaLayoutSnapshot, journal: &OllamaTransactionJournal) -> RecoveryDecision {
    let m = mask(s);
    if m & (2 | 32 | 64) != 0 {
        return defer();
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
            _ => defer(),
        },
        OllamaJournalState::PendingValidation { target, previous } => match m {
            9 if exact_mask(s, 9, &[(1, target), (8, previous)]) => {
                RecoveryDecision::ResumeTargetValidation
            }
            8 if exact_mask(s, 8, &[(8, previous)]) => RecoveryDecision::ResumeRollback,
            _ => defer(),
        },
        OllamaJournalState::CleanupPending { target, previous } => match m {
            9 if exact_mask(s, 9, &[(1, target), (8, previous)]) => RecoveryDecision::ResumeCleanup,
            129 if exact_mask(s, 129, &[(1, target), (128, previous)]) => {
                RecoveryDecision::ResumeCleanup
            }
            1 if exact_mask(s, 1, &[(1, target)]) => RecoveryDecision::RemoveCompletedLegacyJournal,
            8 if exact_mask(s, 8, &[(8, previous)]) => RecoveryDecision::ResumeRollback,
            _ => defer(),
        },
        OllamaJournalState::RollbackPending {
            previous,
            rejected_target,
        } => match rejected_target {
            None => match m {
                0 | 1 if m == 0 || exact_mask(s, 1, &[(1, previous)]) => {
                    RecoveryDecision::RemoveCompletedLegacyJournal
                }
                8 if exact_mask(s, 8, &[(8, previous)]) => RecoveryDecision::ResumeRollback,
                _ => defer(),
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
                _ => defer(),
            },
        },
        OllamaJournalState::RollbackCleanupPending {
            previous,
            rejected_target,
        } => match rejected_target {
            None => {
                if m == 0 || exact_mask(s, 1, &[(1, previous)]) {
                    RecoveryDecision::RemoveCompletedLegacyJournal
                } else {
                    defer()
                }
            }
            Some(rejected) => match m {
                17 if exact_mask(s, 17, &[(1, previous), (16, rejected)]) => {
                    RecoveryDecision::ResumeRollbackCleanup
                }
                257 if exact_mask(s, 257, &[(1, previous), (256, rejected)]) => {
                    RecoveryDecision::ResumeRollbackCleanup
                }
                1 if exact_mask(s, 1, &[(1, previous)]) => {
                    RecoveryDecision::RemoveCompletedLegacyJournal
                }
                _ => defer(),
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
