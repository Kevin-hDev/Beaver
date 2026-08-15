use super::cleanup::{choose, cleanup_state, CleanupTransition};
use super::fingerprint::{BundleFingerprint, OllamaVersion, Sha256Digest};
use super::journal::{OllamaJournalState, OllamaTransactionJournal};
use super::recovery_decision::{
    DirectoryEvidence, JournalPresence, MigrationMarkerPresence, OllamaLayoutSnapshot,
};

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
fn backup_moves_to_distinct_delete_trash_before_recursive_removal() {
    let mut snapshot = empty();
    snapshot.backup = known("1.2.2", "22");
    assert_eq!(choose(&snapshot), Ok(CleanupTransition::MoveBackupToDelete));
    snapshot.backup = DirectoryEvidence::Absent;
    snapshot.backup_delete = known("1.2.2", "22");
    assert_eq!(choose(&snapshot), Ok(CleanupTransition::RemoveBackupDelete));
}

#[test]
fn failed_trash_has_a_different_path_and_two_pass_convergence() {
    let mut snapshot = empty();
    snapshot.failed = known("1.2.3", "33");
    assert_eq!(choose(&snapshot), Ok(CleanupTransition::MoveFailedToDelete));
    snapshot.failed = DirectoryEvidence::Absent;
    snapshot.failed_delete = known("1.2.3", "33");
    assert_eq!(choose(&snapshot), Ok(CleanupTransition::RemoveFailedDelete));
}

#[test]
fn source_and_rebut_are_never_deleted_together() {
    let mut snapshot = empty();
    snapshot.backup = known("1.2.2", "22");
    snapshot.backup_delete = known("1.2.2", "22");
    assert!(choose(&snapshot).is_err());
    snapshot = empty();
    snapshot.failed = known("1.2.3", "33");
    snapshot.failed_delete = known("1.2.3", "33");
    assert!(choose(&snapshot).is_err());
}

#[test]
fn cleanup_pending_is_written_from_pending_validation_once() {
    let journal = OllamaTransactionJournal::new(OllamaJournalState::PendingValidation {
        target: fp("1.2.3", "11"),
        previous: fp("1.2.2", "22"),
    });
    let (_, next) = cleanup_state(&journal).unwrap();
    assert!(matches!(next, OllamaJournalState::CleanupPending { .. }));
}
