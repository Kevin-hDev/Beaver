use super::error::OllamaErrorCode;
use super::fingerprint::{BundleFingerprint, OllamaVersion, Sha256Digest};
use super::journal::{OllamaJournalState, OllamaTransactionJournal};
use super::recovery_decision::{
    decide_recovery, ArchiveDirectoryEvidence, DirectoryEvidence, JournalPresence,
    MigrationMarkerPresence, OllamaLayoutSnapshot, RecoveryDecision,
};

fn fp(version: &str, byte: &str) -> BundleFingerprint {
    BundleFingerprint {
        version: OllamaVersion::parse(version).expect("version"),
        executable_sha256: Sha256Digest::from_hex(&byte.repeat(32)).expect("sha"),
    }
}

fn empty() -> OllamaLayoutSnapshot {
    OllamaLayoutSnapshot {
        journal: JournalPresence::Absent,
        migration_marker: MigrationMarkerPresence::Absent,
        active: DirectoryEvidence::Absent,
        install_staging: DirectoryEvidence::Absent,
        archive_staging: ArchiveDirectoryEvidence::Absent,
        archive_failed: ArchiveDirectoryEvidence::Absent,
        update_staging: DirectoryEvidence::Absent,
        backup: DirectoryEvidence::Absent,
        failed: DirectoryEvidence::Absent,
        legacy_staging: DirectoryEvidence::Absent,
        legacy_backup: DirectoryEvidence::Absent,
        backup_delete: DirectoryEvidence::Absent,
        failed_delete: DirectoryEvidence::Absent,
    }
}

fn journal(state: OllamaJournalState) -> JournalPresence {
    JournalPresence::Valid(OllamaTransactionJournal::new(state))
}

fn known(version: &str, byte: &str) -> DirectoryEvidence {
    DirectoryEvidence::Present(fp(version, byte))
}

fn assert_deferred(state: &OllamaLayoutSnapshot, code: OllamaErrorCode) {
    assert_eq!(decide_recovery(state), RecoveryDecision::Defer { code });
}

#[test]
fn no_journal_and_no_layout_is_ready() {
    assert_eq!(decide_recovery(&empty()), RecoveryDecision::Ready);
}

#[test]
fn install_staging_alone_is_removable() {
    let mut state = empty();
    state.install_staging = known("1.2.3", "11");
    assert_eq!(
        decide_recovery(&state),
        RecoveryDecision::RemoveUncommittedInstallStaging
    );
}

#[test]
fn active_and_install_staging_are_ambiguous() {
    let mut state = empty();
    state.active = known("1.2.3", "11");
    state.install_staging = known("1.2.4", "22");
    assert_deferred(&state, OllamaErrorCode::OllamaRecoveryDeferred);
}

#[test]
fn unreadable_journal_defers_without_layout_decision() {
    let mut state = empty();
    state.journal = JournalPresence::Invalid;
    assert_deferred(&state, OllamaErrorCode::OllamaJournalInvalid);
}

#[test]
fn prepared_journal_with_old_active_and_staging_defers() {
    let target = fp("1.2.3", "11");
    let mut state = empty();
    state.journal = journal(OllamaJournalState::Prepared {
        target: target.clone(),
        previous: fp("1.2.2", "22"),
    });
    state.active = DirectoryEvidence::Present(target.clone());
    state.update_staging = DirectoryEvidence::Present(target);
    assert_deferred(&state, OllamaErrorCode::OllamaRecoveryDeferred);
}

#[test]
fn pending_validation_with_exact_target_and_backup_resumes_validation() {
    let target = fp("1.2.3", "11");
    let previous = fp("1.2.2", "22");
    let mut state = empty();
    state.journal = journal(OllamaJournalState::PendingValidation {
        target: target.clone(),
        previous: previous.clone(),
    });
    state.active = DirectoryEvidence::Present(target);
    state.backup = DirectoryEvidence::Present(previous);
    assert_eq!(
        decide_recovery(&state),
        RecoveryDecision::ResumeTargetValidation
    );
}

#[test]
fn pending_validation_without_backup_is_ambiguous() {
    let mut state = empty();
    state.journal = journal(OllamaJournalState::PendingValidation {
        target: fp("1.2.3", "11"),
        previous: fp("1.2.2", "22"),
    });
    state.active = known("1.2.3", "11");
    assert_deferred(&state, OllamaErrorCode::OllamaRecoveryDeferred);
}

#[test]
fn cleanup_pending_with_exact_target_and_backup_resumes_cleanup() {
    let target = fp("1.2.3", "11");
    let previous = fp("1.2.2", "22");
    let mut state = empty();
    state.journal = journal(OllamaJournalState::CleanupPending {
        target: target.clone(),
        previous: previous.clone(),
    });
    state.active = DirectoryEvidence::Present(target);
    state.backup = DirectoryEvidence::Present(previous);
    assert_eq!(decide_recovery(&state), RecoveryDecision::ResumeCleanup);
}

#[test]
fn rollback_pending_without_rejected_target_and_previous_only_removes_journal() {
    let mut state = empty();
    state.journal = journal(OllamaJournalState::RollbackPending {
        previous: fp("1.2.2", "22"),
        rejected_target: None,
    });
    assert_eq!(
        decide_recovery(&state),
        RecoveryDecision::RemoveCompletedLegacyJournal
    );
}

#[test]
fn rollback_pending_with_rejected_target_without_rebut_is_ambiguous() {
    let mut state = empty();
    state.journal = journal(OllamaJournalState::RollbackPending {
        previous: fp("1.2.2", "22"),
        rejected_target: Some(fp("1.2.3", "11")),
    });
    assert_deferred(&state, OllamaErrorCode::OllamaRecoveryDeferred);
}

#[test]
fn rollback_cleanup_pending_without_rejected_target_and_previous_only_removes_journal() {
    let mut state = empty();
    state.journal = journal(OllamaJournalState::RollbackCleanupPending {
        previous: fp("1.2.2", "22"),
        rejected_target: None,
    });
    assert_eq!(
        decide_recovery(&state),
        RecoveryDecision::RemoveCompletedLegacyJournal
    );
}

#[test]
fn failed_or_failed_delete_without_rejected_target_stays_ambiguous() {
    for failed in [false, true] {
        let mut state = empty();
        state.journal = journal(OllamaJournalState::RollbackCleanupPending {
            previous: fp("1.2.2", "22"),
            rejected_target: None,
        });
        if failed {
            state.failed = known("1.2.3", "11");
        } else {
            state.failed_delete = known("1.2.3", "11");
        }
        assert_deferred(&state, OllamaErrorCode::OllamaRecoveryDeferred);
    }
}

#[test]
fn legacy_active_is_adopted_before_durable_marker() {
    let mut state = empty();
    state.active = known("1.2.3", "11");
    assert_eq!(decide_recovery(&state), RecoveryDecision::AdoptLegacyActive);
}

#[test]
fn legacy_backup_without_active_is_restored_before_marker() {
    let mut state = empty();
    state.legacy_backup = known("1.2.2", "22");
    assert_eq!(
        decide_recovery(&state),
        RecoveryDecision::RestoreLegacyBackup
    );
}

#[test]
fn legacy_backup_after_durable_marker_is_ambiguous_and_untouched() {
    let mut state = empty();
    state.migration_marker = MigrationMarkerPresence::Valid(Default::default());
    state.legacy_backup = known("1.2.2", "22");
    assert_deferred(&state, OllamaErrorCode::OllamaRecoveryDeferred);
}

#[test]
fn modern_backup_is_the_only_backup_after_marker() {
    let mut state = empty();
    state.migration_marker = MigrationMarkerPresence::Valid(Default::default());
    state.backup = known("1.2.2", "22");
    assert_eq!(
        decide_recovery(&state),
        RecoveryDecision::RestoreLegacyBackup
    );
}

#[test]
fn unknown_staging_and_different_fingerprint_defer() {
    let mut unknown = empty();
    unknown.update_staging = DirectoryEvidence::Unknown;
    assert_deferred(&unknown, OllamaErrorCode::OllamaRecoveryDeferred);
    let mut mismatch = empty();
    mismatch.journal = journal(OllamaJournalState::PendingValidation {
        target: fp("1.2.3", "11"),
        previous: fp("1.2.2", "22"),
    });
    mismatch.active = known("1.2.9", "99");
    mismatch.backup = known("1.2.2", "22");
    assert_deferred(&mismatch, OllamaErrorCode::OllamaRecoveryDeferred);
}

#[test]
fn all_fixed_ten_presence_combinations_are_total_and_bounded() {
    let target = fp("1.2.3", "11");
    let previous = fp("1.2.2", "22");
    for mask in 0_u16..(1_u16 << 10) {
        let mut state = empty();
        if mask & 1 != 0 {
            state.journal = journal(OllamaJournalState::PendingValidation {
                target: target.clone(),
                previous: previous.clone(),
            });
        }
        let evidence = |bit: u32| {
            if mask & (1_u16 << bit) != 0 {
                DirectoryEvidence::Present(target.clone())
            } else {
                DirectoryEvidence::Absent
            }
        };
        state.active = evidence(1);
        state.install_staging = evidence(2);
        state.update_staging = evidence(3);
        state.backup = if mask & (1_u16 << 4) != 0 {
            DirectoryEvidence::Present(previous.clone())
        } else {
            DirectoryEvidence::Absent
        };
        state.failed = evidence(5);
        state.legacy_staging = evidence(6);
        state.legacy_backup = evidence(7);
        state.backup_delete = evidence(8);
        state.failed_delete = evidence(9);
        let decision = decide_recovery(&state);
        let ambiguous = state.active != DirectoryEvidence::Absent
            && (state.backup != DirectoryEvidence::Absent
                || state.install_staging != DirectoryEvidence::Absent);
        assert!(!(ambiguous && matches!(decision, RecoveryDecision::Ready)));
    }
}

#[test]
fn no_journal_active_and_modern_update_staging_only_removes_staging() {
    let mut state = empty();
    state.active = known("1.2.3", "11");
    state.update_staging = known("1.2.4", "22");
    assert_eq!(
        decide_recovery(&state),
        RecoveryDecision::RemoveUncommittedInstallStaging
    );
}

#[test]
fn no_journal_active_and_legacy_staging_only_removes_staging_before_marker() {
    let mut state = empty();
    state.active = known("1.2.3", "11");
    state.legacy_staging = known("1.2.4", "22");
    assert_eq!(
        decide_recovery(&state),
        RecoveryDecision::RemoveUncommittedInstallStaging
    );
}

#[test]
fn no_journal_active_and_legacy_backup_creates_validation_recovery() {
    let mut state = empty();
    state.active = known("1.2.3", "11");
    state.legacy_backup = known("1.2.2", "22");
    assert_eq!(
        decide_recovery(&state),
        RecoveryDecision::ResumeTargetValidation
    );
}

#[test]
fn no_journal_active_and_failed_creates_rollback_cleanup_recovery() {
    let mut state = empty();
    state.active = known("1.2.2", "22");
    state.failed = known("1.2.3", "11");
    assert_eq!(
        decide_recovery(&state),
        RecoveryDecision::ResumeRollbackCleanup
    );
}

#[test]
fn no_journal_failed_or_rebut_alone_is_ambiguous_and_unchanged() {
    for failed in [true, false] {
        let mut state = empty();
        if failed {
            state.failed = known("1.2.3", "11");
        } else {
            state.failed_delete = known("1.2.3", "11");
        }
        let before = state.clone();
        assert_deferred(&state, OllamaErrorCode::OllamaRecoveryDeferred);
        assert_eq!(state, before);
    }
}

#[test]
fn prepared_previous_and_target_staging_commits_fresh_install() {
    let mut state = empty();
    state.journal = journal(OllamaJournalState::Prepared {
        target: fp("1.2.4", "22"),
        previous: fp("1.2.3", "11"),
    });
    state.active = known("1.2.3", "11");
    state.update_staging = known("1.2.4", "22");
    assert_eq!(
        decide_recovery(&state),
        RecoveryDecision::CommitFreshInstall
    );
}

#[test]
fn prepared_backup_and_target_staging_finish_fresh_install_commit() {
    let mut state = empty();
    state.journal = journal(OllamaJournalState::Prepared {
        target: fp("1.2.4", "22"),
        previous: fp("1.2.3", "11"),
    });
    state.backup = known("1.2.3", "11");
    state.update_staging = known("1.2.4", "22");
    assert_eq!(
        decide_recovery(&state),
        RecoveryDecision::CommitFreshInstall
    );
}

#[test]
fn prepared_target_and_backup_without_staging_resume_validation() {
    let mut state = empty();
    state.journal = journal(OllamaJournalState::Prepared {
        target: fp("1.2.4", "22"),
        previous: fp("1.2.3", "11"),
    });
    state.active = known("1.2.4", "22");
    state.backup = known("1.2.3", "11");
    assert_eq!(
        decide_recovery(&state),
        RecoveryDecision::ResumeTargetValidation
    );
}

#[test]
fn prepared_previous_only_removes_abandoned_journal() {
    let mut state = empty();
    state.journal = journal(OllamaJournalState::Prepared {
        target: fp("1.2.4", "22"),
        previous: fp("1.2.3", "11"),
    });
    state.active = known("1.2.3", "11");
    assert_eq!(
        decide_recovery(&state),
        RecoveryDecision::RemoveCompletedLegacyJournal
    );
}

#[test]
fn prepared_backup_only_resumes_rollback() {
    let mut state = empty();
    state.journal = journal(OllamaJournalState::Prepared {
        target: fp("1.2.4", "22"),
        previous: fp("1.2.3", "11"),
    });
    state.backup = known("1.2.3", "11");
    assert_eq!(decide_recovery(&state), RecoveryDecision::ResumeRollback);
}

#[test]
fn pending_validation_backup_only_resumes_rollback() {
    let mut state = empty();
    state.journal = journal(OllamaJournalState::PendingValidation {
        target: fp("1.2.4", "22"),
        previous: fp("1.2.3", "11"),
    });
    state.backup = known("1.2.3", "11");
    assert_eq!(decide_recovery(&state), RecoveryDecision::ResumeRollback);
}

#[test]
fn cleanup_pending_rebut_only_resumes_cleanup() {
    let mut state = empty();
    state.journal = journal(OllamaJournalState::CleanupPending {
        target: fp("1.2.4", "22"),
        previous: fp("1.2.3", "11"),
    });
    state.active = known("1.2.4", "22");
    state.backup_delete = known("1.2.3", "11");
    assert_eq!(decide_recovery(&state), RecoveryDecision::ResumeCleanup);
}

#[test]
fn cleanup_pending_target_only_removes_completed_journal() {
    let mut state = empty();
    state.journal = journal(OllamaJournalState::CleanupPending {
        target: fp("1.2.4", "22"),
        previous: fp("1.2.3", "11"),
    });
    state.active = known("1.2.4", "22");
    assert_eq!(
        decide_recovery(&state),
        RecoveryDecision::RemoveCompletedLegacyJournal
    );
}

#[test]
fn cleanup_pending_backup_only_resumes_rollback() {
    let mut state = empty();
    state.journal = journal(OllamaJournalState::CleanupPending {
        target: fp("1.2.4", "22"),
        previous: fp("1.2.3", "11"),
    });
    state.backup = known("1.2.3", "11");
    assert_eq!(decide_recovery(&state), RecoveryDecision::ResumeRollback);
}

#[test]
fn rollback_pending_rejected_target_and_backup_resume_rollback() {
    let mut state = empty();
    state.journal = journal(OllamaJournalState::RollbackPending {
        previous: fp("1.2.3", "11"),
        rejected_target: Some(fp("1.2.4", "22")),
    });
    state.active = known("1.2.4", "22");
    state.backup = known("1.2.3", "11");
    assert_eq!(decide_recovery(&state), RecoveryDecision::ResumeRollback);
}

#[test]
fn rollback_pending_rejected_rebut_and_backup_resume_rollback() {
    let mut state = empty();
    state.journal = journal(OllamaJournalState::RollbackPending {
        previous: fp("1.2.3", "11"),
        rejected_target: Some(fp("1.2.4", "22")),
    });
    state.backup = known("1.2.3", "11");
    state.failed = known("1.2.4", "22");
    assert_eq!(decide_recovery(&state), RecoveryDecision::ResumeRollback);
}

#[test]
fn rollback_pending_restored_previous_and_rejected_target_resume_cleanup() {
    let mut state = empty();
    state.journal = journal(OllamaJournalState::RollbackPending {
        previous: fp("1.2.3", "11"),
        rejected_target: Some(fp("1.2.4", "22")),
    });
    state.active = known("1.2.3", "11");
    state.failed = known("1.2.4", "22");
    assert_eq!(
        decide_recovery(&state),
        RecoveryDecision::ResumeRollbackCleanup
    );
}

#[test]
fn rollback_cleanup_rejected_rebut_or_partial_rebut_resume_cleanup() {
    for failed_delete in [false, true] {
        let mut state = empty();
        state.journal = journal(OllamaJournalState::RollbackCleanupPending {
            previous: fp("1.2.3", "11"),
            rejected_target: Some(fp("1.2.4", "22")),
        });
        state.active = known("1.2.3", "11");
        if failed_delete {
            state.failed_delete = known("1.2.4", "22");
        } else {
            state.failed = known("1.2.4", "22");
        }
        assert_eq!(
            decide_recovery(&state),
            RecoveryDecision::ResumeRollbackCleanup
        );
    }
}

#[test]
fn durable_marker_changes_legacy_staging_from_cleanup_to_defer() {
    let mut state = empty();
    state.migration_marker = MigrationMarkerPresence::Valid(Default::default());
    state.active = known("1.2.3", "11");
    state.legacy_staging = known("1.2.4", "22");
    let before = state.clone();
    assert_deferred(&state, OllamaErrorCode::OllamaRecoveryDeferred);
    assert_eq!(state, before);
}

#[test]
fn temporary_marker_keeps_legacy_rules_active() {
    let mut state = empty();
    state.migration_marker = MigrationMarkerPresence::Temporary;
    state.legacy_backup = known("1.2.2", "22");
    assert_eq!(
        decide_recovery(&state),
        RecoveryDecision::RestoreLegacyBackup
    );
}
