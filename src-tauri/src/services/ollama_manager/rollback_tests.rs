use super::fingerprint::{BundleFingerprint, OllamaVersion, Sha256Digest};
use super::journal::{OllamaJournalState, OllamaTransactionJournal};
use super::recovery_decision::{
    ArchiveDirectoryEvidence, DirectoryEvidence, JournalPresence, MigrationMarkerPresence,
    OllamaLayoutSnapshot,
};
use super::rollback::{choose, cleanup_state, rejected_state, RollbackTransition};

fn fp(version: &str, byte: &str) -> BundleFingerprint {
    BundleFingerprint {
        version: OllamaVersion::parse(version).unwrap(),
        executable_sha256: Sha256Digest::from_hex(&byte.repeat(32)).unwrap(),
    }
}
fn known(version: &str, byte: &str) -> DirectoryEvidence {
    DirectoryEvidence::Present(fp(version, byte))
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

#[test]
fn rejected_target_moves_to_failed_and_never_directly_disappears() {
    let mut snapshot = empty();
    snapshot.active = known("1.2.3", "33");
    assert_eq!(
        choose(&snapshot),
        Ok(RollbackTransition::MoveRejectedToFailed)
    );
}

#[test]
fn previous_bundle_restores_before_failed_rebut_cleanup() {
    let mut snapshot = empty();
    snapshot.backup = known("1.2.2", "22");
    assert_eq!(choose(&snapshot), Ok(RollbackTransition::RestorePrevious));
    snapshot.backup = DirectoryEvidence::Absent;
    snapshot.active = known("1.2.2", "22");
    snapshot.failed = known("1.2.3", "33");
    assert_eq!(
        choose(&snapshot),
        Ok(RollbackTransition::PersistRollbackCleanupPending)
    );
}

#[test]
fn rollback_cleanup_rejects_source_and_rebut_aliases() {
    let mut snapshot = empty();
    snapshot.active = known("1.2.2", "22");
    snapshot.failed = known("1.2.3", "33");
    snapshot.failed_delete = known("1.2.3", "33");
    assert!(choose(&snapshot).is_err());
}

#[test]
fn rejected_target_and_cleanup_phases_are_durable_and_idempotent() {
    let journal = OllamaTransactionJournal::new(OllamaJournalState::PendingValidation {
        target: fp("1.2.3", "33"),
        previous: fp("1.2.2", "22"),
    });
    let rollback = rejected_state(&journal).unwrap();
    assert!(matches!(
        rollback,
        OllamaJournalState::RollbackPending {
            rejected_target: Some(_),
            ..
        }
    ));
    let cleanup = cleanup_state(&OllamaTransactionJournal::new(rollback)).unwrap();
    assert!(matches!(
        cleanup,
        OllamaJournalState::RollbackCleanupPending {
            rejected_target: Some(_),
            ..
        }
    ));
}

#[test]
fn active_and_backup_allow_rejected_target_to_move_to_failed_first() {
    let mut snapshot = empty();
    snapshot.active = known("1.2.3", "33");
    snapshot.backup = known("1.2.2", "22");
    assert_eq!(
        choose(&snapshot),
        Ok(RollbackTransition::MoveRejectedToFailed)
    );
}
