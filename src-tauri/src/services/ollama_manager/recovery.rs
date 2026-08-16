#![allow(dead_code)]

use super::cleanup;
use super::durable_fs::OllamaDurableFs;
use super::error::OllamaErrorCode;
use super::journal::OllamaTransactionJournal;
use super::journal_store::OllamaJournalStore;
use super::path_identity::CanonicalDirectory;
use super::recovery_decision::{
    decide_recovery, JournalPresence, OllamaLayoutSnapshot, RecoveryDecision,
};
use super::recovery_helpers::{present, target_of};
pub(crate) use super::recovery_probe::{RecoveryProbe, RecoveryProbeResult};
use super::recovery_types::ApplyResult;
pub use super::recovery_types::{RecoveryOutcome, RecoveryReason};
use super::rollback;
use crate::services::paths::OllamaPaths;
use std::sync::Arc;
pub struct RecoveryExecutor<F, P>
where
    F: OllamaDurableFs + 'static,
    P: RecoveryProbe + 'static,
{
    pub(crate) fs: Arc<F>,
    pub(crate) probe: Arc<P>,
    pub(crate) journal: OllamaJournalStore<F>,
    paths: OllamaPaths,
    models_directory: Option<CanonicalDirectory>,
}

impl<F, P> RecoveryExecutor<F, P>
where
    F: OllamaDurableFs + 'static,
    P: RecoveryProbe + 'static,
{
    pub(crate) fn new(fs: Arc<F>, probe: Arc<P>, paths: OllamaPaths) -> Self {
        Self {
            journal: OllamaJournalStore::new(Arc::clone(&fs), paths.clone()),
            fs,
            probe,
            paths,
            models_directory: None,
        }
    }

    pub(crate) fn new_with_models(
        fs: Arc<F>,
        probe: Arc<P>,
        paths: OllamaPaths,
        models_directory: CanonicalDirectory,
    ) -> Self {
        let mut executor = Self::new(fs, probe, paths);
        executor.models_directory = Some(models_directory);
        executor
    }

    pub(crate) async fn recover(
        &self,
        reason: RecoveryReason,
    ) -> Result<RecoveryOutcome, OllamaErrorCode> {
        if super::temporary_recovery::remove_one(&self.fs, &self.journal, &self.paths).await? {
            return Ok(RecoveryOutcome::ProgressMade);
        }
        let journal = self
            .journal
            .read()
            .await?
            .map_or(JournalPresence::Absent, JournalPresence::Valid);
        let snapshot = cleanup::snapshot(journal, &*self.fs, &self.paths);
        let outcome = self.execute_snapshot(&snapshot, reason).await?;
        if matches!(outcome, RecoveryOutcome::ProgressMade) {
            let journal = self
                .journal
                .read()
                .await?
                .map_or(JournalPresence::Absent, JournalPresence::Valid);
            let _ = cleanup::snapshot(journal, &*self.fs, &self.paths);
        }
        Ok(outcome)
    }

    pub(crate) async fn execute_snapshot(
        &self,
        snapshot: &OllamaLayoutSnapshot,
        _reason: RecoveryReason,
    ) -> Result<RecoveryOutcome, OllamaErrorCode> {
        if let Some(action) = super::staging_recovery::decide(&snapshot.journal, &self.paths)? {
            super::staging_recovery::apply(
                action,
                &self.fs,
                &self.paths,
                self.models_directory.as_ref(),
            )
            .await?;
            return Ok(RecoveryOutcome::ProgressMade);
        }
        if let Some(action) = super::archive_recovery::decide(snapshot)? {
            super::archive_recovery::apply(
                action,
                &self.fs,
                &self.paths,
                self.models_directory.as_ref(),
            )
            .await?;
            return Ok(RecoveryOutcome::ProgressMade);
        }
        match decide_recovery(snapshot) {
            RecoveryDecision::Ready => Ok(RecoveryOutcome::Ready),
            RecoveryDecision::Defer { code } => Ok(RecoveryOutcome::Deferred { code }),
            decision => match self.apply(snapshot, decision).await? {
                ApplyResult::Progress => Ok(RecoveryOutcome::ProgressMade),
                ApplyResult::Deferred(code) => Ok(RecoveryOutcome::Deferred { code }),
            },
        }
    }

    async fn apply(
        &self,
        snapshot: &OllamaLayoutSnapshot,
        decision: RecoveryDecision,
    ) -> Result<ApplyResult, OllamaErrorCode> {
        match decision {
            RecoveryDecision::CommitFreshInstall => self.commit(snapshot).await,
            RecoveryDecision::ResumeTargetValidation => self.validate_target(snapshot).await,
            RecoveryDecision::ResumeCleanup => {
                let transition = cleanup::choose(snapshot)?;
                cleanup::apply(
                    transition,
                    &self.fs,
                    &self.journal,
                    &self.paths,
                    self.models_directory.as_ref(),
                )
                .await?;
                Ok(ApplyResult::Progress)
            }
            RecoveryDecision::ResumeRollback | RecoveryDecision::ResumeRollbackCleanup => {
                let transition = rollback::choose(snapshot)?;
                rollback::apply(
                    transition,
                    snapshot,
                    &self.fs,
                    &self.journal,
                    &self.paths,
                    self.models_directory.as_ref(),
                )
                .await?;
                Ok(ApplyResult::Progress)
            }
            RecoveryDecision::AdoptLegacyActive => {
                let expected = match &snapshot.active {
                    super::recovery_decision::DirectoryEvidence::Present(value) => value,
                    _ => return Err(OllamaErrorCode::OllamaUpdateRecoveryRequired),
                };
                super::adoption::validate_and_sync(&self.fs, &self.paths, expected).await?;
                cleanup::write_marker(&self.fs, &self.paths).await?;
                Ok(ApplyResult::Progress)
            }
            RecoveryDecision::RestoreLegacyBackup => {
                let source = if present(&snapshot.legacy_backup) {
                    &self.paths.legacy_backup
                } else {
                    &self.paths.backup
                };
                cleanup::rename(&self.fs, source, &self.paths.active).await?;
                Ok(ApplyResult::Progress)
            }
            RecoveryDecision::RemoveCompletedLegacyJournal => {
                self.journal.remove().await?;
                Ok(ApplyResult::Progress)
            }
            RecoveryDecision::Ready | RecoveryDecision::Defer { .. } => unreachable!(),
        }
    }

    async fn commit(
        &self,
        snapshot: &OllamaLayoutSnapshot,
    ) -> Result<ApplyResult, OllamaErrorCode> {
        let (source, destination) =
            if present(&snapshot.active) && present(&snapshot.update_staging) {
                (&self.paths.active, &self.paths.backup)
            } else if present(&snapshot.update_staging) {
                (&self.paths.update_staging, &self.paths.active)
            } else {
                (&self.paths.install_staging, &self.paths.active)
            };
        cleanup::rename(&self.fs, source, destination).await?;
        Ok(ApplyResult::Progress)
    }

    async fn validate_target(
        &self,
        snapshot: &OllamaLayoutSnapshot,
    ) -> Result<ApplyResult, OllamaErrorCode> {
        let journal = match &snapshot.journal {
            JournalPresence::Valid(value) => value.clone(),
            JournalPresence::Invalid => return Err(OllamaErrorCode::OllamaJournalInvalid),
            _ => return Err(OllamaErrorCode::OllamaRecoveryDeferred),
        };
        let target = target_of(&journal).ok_or(OllamaErrorCode::OllamaJournalInvalid)?;
        match self.probe.validate(target, &self.paths).await {
            RecoveryProbeResult::Valid => {
                let state = cleanup::cleanup_state(&journal)
                    .ok_or(OllamaErrorCode::OllamaJournalInvalid)?
                    .1;
                self.journal
                    .replace(&OllamaTransactionJournal::new(state))
                    .await?;
                Ok(ApplyResult::Progress)
            }
            RecoveryProbeResult::Invalid(_code) => {
                let state = rollback::rejected_state(&journal)
                    .ok_or(OllamaErrorCode::OllamaJournalInvalid)?;
                self.journal
                    .replace(&OllamaTransactionJournal::new(state))
                    .await?;
                Ok(ApplyResult::Progress)
            }
            RecoveryProbeResult::Deferred(code) => Ok(ApplyResult::Deferred(code)),
        }
    }
}
