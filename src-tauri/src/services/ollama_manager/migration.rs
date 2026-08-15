#![allow(dead_code)]

use super::recovery_decision::MigrationMarkerPresence;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BackupPolicy {
    LegacyAndModern,
    ModernOnly,
    Ambiguous,
}

pub fn classify_backup_policy(marker: &MigrationMarkerPresence) -> BackupPolicy {
    match marker {
        MigrationMarkerPresence::Absent | MigrationMarkerPresence::Temporary => {
            BackupPolicy::LegacyAndModern
        }
        MigrationMarkerPresence::Valid(_) => BackupPolicy::ModernOnly,
        MigrationMarkerPresence::Invalid | MigrationMarkerPresence::Unknown => {
            BackupPolicy::Ambiguous
        }
    }
}
