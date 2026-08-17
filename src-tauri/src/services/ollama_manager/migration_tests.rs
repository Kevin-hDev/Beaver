use super::migration::{classify_backup_policy, BackupPolicy};
use super::recovery_decision::MigrationMarkerPresence;

#[test]
fn absent_marker_keeps_legacy_backup_legal() {
    assert_eq!(
        classify_backup_policy(&MigrationMarkerPresence::Absent),
        BackupPolicy::LegacyAndModern
    );
}

#[test]
fn valid_marker_makes_legacy_ambiguous_and_modern_legal() {
    assert_eq!(
        classify_backup_policy(&MigrationMarkerPresence::Valid(Default::default())),
        BackupPolicy::ModernOnly
    );
}

#[test]
fn invalid_or_unknown_marker_blocks_both_backup_names() {
    for marker in [
        MigrationMarkerPresence::Invalid,
        MigrationMarkerPresence::Unknown,
    ] {
        assert_eq!(classify_backup_policy(&marker), BackupPolicy::Ambiguous);
    }
}

#[test]
fn marker_temporary_state_is_not_an_authority() {
    assert_eq!(
        classify_backup_policy(&MigrationMarkerPresence::Absent),
        BackupPolicy::LegacyAndModern
    );
}
