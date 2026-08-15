#![allow(dead_code)]

use super::error::OllamaErrorCode;
use super::fingerprint::BundleFingerprint;
use super::journal::{OllamaMigrationMarker, OllamaTransactionJournal};
use super::migration::{classify_backup_policy, BackupPolicy};
use super::recovery_decision_rules;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DirectoryEvidence {
    Absent,
    Present(BundleFingerprint),
    Unknown,
    Invalid,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArchiveDirectoryEvidence {
    Absent,
    Present,
    Unknown,
    Invalid,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum JournalPresence {
    Absent,
    Valid(OllamaTransactionJournal),
    Invalid,
    Unknown,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MigrationMarkerPresence {
    Absent,
    Temporary,
    Valid(OllamaMigrationMarker),
    Invalid,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OllamaLayoutSnapshot {
    pub journal: JournalPresence,
    pub migration_marker: MigrationMarkerPresence,
    pub active: DirectoryEvidence,
    pub install_staging: DirectoryEvidence,
    pub archive_staging: ArchiveDirectoryEvidence,
    pub archive_failed: ArchiveDirectoryEvidence,
    pub update_staging: DirectoryEvidence,
    pub backup: DirectoryEvidence,
    pub failed: DirectoryEvidence,
    pub legacy_staging: DirectoryEvidence,
    pub legacy_backup: DirectoryEvidence,
    pub backup_delete: DirectoryEvidence,
    pub failed_delete: DirectoryEvidence,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecoveryDecision {
    Ready,
    CommitFreshInstall,
    RemoveUncommittedInstallStaging,
    ResumeTargetValidation,
    ResumeCleanup,
    ResumeRollback,
    ResumeRollbackCleanup,
    AdoptLegacyActive,
    RestoreLegacyBackup,
    RemoveCompletedLegacyJournal,
    Defer { code: OllamaErrorCode },
}

pub(super) const fn defer() -> RecoveryDecision {
    RecoveryDecision::Defer {
        code: OllamaErrorCode::OllamaRecoveryDeferred,
    }
}
fn is_present(e: &DirectoryEvidence) -> bool {
    matches!(e, DirectoryEvidence::Present(_))
}
fn is_exact(e: &DirectoryEvidence, fp: &BundleFingerprint) -> bool {
    matches!(e, DirectoryEvidence::Present(actual) if actual == fp)
}
pub(super) fn mask(s: &OllamaLayoutSnapshot) -> u16 {
    let dirs = [
        &s.active,
        &s.install_staging,
        &s.update_staging,
        &s.backup,
        &s.failed,
        &s.legacy_staging,
        &s.legacy_backup,
        &s.backup_delete,
        &s.failed_delete,
    ];
    dirs.iter().enumerate().fold(0, |value, (bit, item)| {
        value | if is_present(item) { 1_u16 << bit } else { 0 }
    })
}
pub(super) fn exact_mask(
    s: &OllamaLayoutSnapshot,
    wanted: u16,
    entries: &[(u16, &BundleFingerprint)],
) -> bool {
    mask(s) == wanted
        && entries.iter().all(|(bit, fp)| match *bit {
            1 => is_exact(&s.active, fp),
            2 => is_exact(&s.install_staging, fp),
            4 => is_exact(&s.update_staging, fp),
            8 => is_exact(&s.backup, fp),
            16 => is_exact(&s.failed, fp),
            32 => is_exact(&s.legacy_staging, fp),
            64 => is_exact(&s.legacy_backup, fp),
            128 => is_exact(&s.backup_delete, fp),
            256 => is_exact(&s.failed_delete, fp),
            _ => false,
        })
}
fn has_unknown(s: &OllamaLayoutSnapshot) -> bool {
    let dirs = [
        &s.active,
        &s.install_staging,
        &s.update_staging,
        &s.backup,
        &s.failed,
        &s.legacy_staging,
        &s.legacy_backup,
        &s.backup_delete,
        &s.failed_delete,
    ];
    dirs.iter()
        .any(|e| matches!(e, DirectoryEvidence::Unknown | DirectoryEvidence::Invalid))
        || matches!(s.journal, JournalPresence::Unknown)
        || matches!(
            s.migration_marker,
            MigrationMarkerPresence::Invalid | MigrationMarkerPresence::Unknown
        )
}
pub fn decide_recovery(s: &OllamaLayoutSnapshot) -> RecoveryDecision {
    if matches!(s.journal, JournalPresence::Invalid) {
        return RecoveryDecision::Defer {
            code: OllamaErrorCode::OllamaJournalInvalid,
        };
    }
    if [s.archive_staging, s.archive_failed]
        .into_iter()
        .any(|evidence| !matches!(evidence, ArchiveDirectoryEvidence::Absent))
    {
        return defer();
    }
    if has_unknown(s)
        || matches!(
            classify_backup_policy(&s.migration_marker),
            BackupPolicy::Ambiguous
        )
    {
        return defer();
    }
    match &s.journal {
        JournalPresence::Absent => recovery_decision_rules::decide_without_journal(
            s,
            classify_backup_policy(&s.migration_marker),
        ),
        JournalPresence::Valid(journal) => recovery_decision_rules::decide_with_journal(s, journal),
        JournalPresence::Invalid | JournalPresence::Unknown => defer(),
    }
}
