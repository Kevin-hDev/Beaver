#![allow(dead_code)]

use super::blocking::run_ollama_blocking;
pub(crate) use super::cleanup_inspection::snapshot;
use super::cleanup_inspection::validate_trash;
use super::durable_fs::OllamaDurableFs;
use super::error::OllamaErrorCode;
use super::journal::{OllamaJournalState, OllamaMigrationMarker, OllamaTransactionJournal};
use super::journal_store::OllamaJournalStore;
use super::path_identity::CanonicalDirectory;
use super::recovery_decision::{DirectoryEvidence, OllamaLayoutSnapshot};
use crate::services::paths::OllamaPaths;
use std::path::Path;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CleanupTransition {
    MoveBackupToDelete,
    RemoveBackupDelete,
    MoveFailedToDelete,
    RemoveFailedDelete,
    RemoveJournal,
}

pub(crate) fn choose(
    snapshot: &OllamaLayoutSnapshot,
) -> Result<CleanupTransition, OllamaErrorCode> {
    if present(&snapshot.backup) && present(&snapshot.backup_delete)
        || present(&snapshot.failed) && present(&snapshot.failed_delete)
    {
        return Err(OllamaErrorCode::OllamaRecoveryDeferred);
    }
    if present(&snapshot.backup) {
        return Ok(CleanupTransition::MoveBackupToDelete);
    }
    if present(&snapshot.backup_delete) {
        return Ok(CleanupTransition::RemoveBackupDelete);
    }
    if present(&snapshot.failed) {
        return Ok(CleanupTransition::MoveFailedToDelete);
    }
    if present(&snapshot.failed_delete) {
        return Ok(CleanupTransition::RemoveFailedDelete);
    }
    Ok(CleanupTransition::RemoveJournal)
}

pub(crate) fn cleanup_state(
    journal: &OllamaTransactionJournal,
) -> Option<(OllamaJournalState, OllamaJournalState)> {
    let (target, previous) = match &journal.state {
        OllamaJournalState::Prepared { target, previous }
        | OllamaJournalState::PendingValidation { target, previous } => {
            (target.clone(), previous.clone())
        }
        _ => return None,
    };
    Some((
        journal.state.clone(),
        OllamaJournalState::CleanupPending { target, previous },
    ))
}

pub(crate) async fn apply<F>(
    transition: CleanupTransition,
    fs: &Arc<F>,
    journal: &OllamaJournalStore<F>,
    paths: &OllamaPaths,
    models: Option<&CanonicalDirectory>,
) -> Result<(), OllamaErrorCode>
where
    F: OllamaDurableFs + 'static,
{
    match transition {
        CleanupTransition::MoveBackupToDelete => {
            rename(fs, &paths.backup, &paths.backup_delete).await
        }
        CleanupTransition::RemoveBackupDelete => {
            remove_trash(fs, &paths.backup_delete, paths, models).await
        }
        CleanupTransition::MoveFailedToDelete => {
            rename(fs, &paths.failed, &paths.failed_delete).await
        }
        CleanupTransition::RemoveFailedDelete => {
            remove_trash(fs, &paths.failed_delete, paths, models).await
        }
        CleanupTransition::RemoveJournal => journal.remove().await,
    }
}

pub(crate) async fn rename<F>(
    fs: &Arc<F>,
    source: &Path,
    destination: &Path,
) -> Result<(), OllamaErrorCode>
where
    F: OllamaDurableFs + 'static,
{
    let fs = Arc::clone(fs);
    let source = source.to_path_buf();
    let destination = destination.to_path_buf();
    run_ollama_blocking(move || {
        fs.rename_durable(&source, &destination)
            .map_err(|_| OllamaErrorCode::OllamaStorageUnavailable)
    })
    .await
}

pub(crate) async fn write_marker<F>(fs: &Arc<F>, paths: &OllamaPaths) -> Result<(), OllamaErrorCode>
where
    F: OllamaDurableFs + 'static,
{
    let bytes = serde_json::to_vec(&OllamaMigrationMarker::new())
        .map_err(|_| OllamaErrorCode::OllamaInternal)?;
    let fs = Arc::clone(fs);
    let paths = paths.clone();
    run_ollama_blocking(move || {
        fs.write_new_atomic(&paths.migration_marker_tmp, &paths.migration_marker, &bytes)
            .map_err(|_| OllamaErrorCode::OllamaStorageUnavailable)
    })
    .await
}

pub(crate) async fn remove_safe_journal_tmp<F>(
    fs: &Arc<F>,
    journal: &OllamaJournalStore<F>,
    paths: &OllamaPaths,
) -> Result<bool, OllamaErrorCode>
where
    F: OllamaDurableFs + 'static,
{
    let metadata = match std::fs::symlink_metadata(&paths.journal_tmp) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(_) => return Err(OllamaErrorCode::OllamaUpdateRecoveryRequired),
    };
    if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() > 4 * 1024 {
        return Err(OllamaErrorCode::OllamaUpdateRecoveryRequired);
    }
    if journal.read().await.is_err() {
        return Err(OllamaErrorCode::OllamaUpdateRecoveryRequired);
    }
    let fs = Arc::clone(fs);
    let path = paths.journal_tmp.clone();
    run_ollama_blocking(move || {
        fs.remove_file_durable(&path)
            .map_err(|_| OllamaErrorCode::OllamaStorageUnavailable)
    })
    .await?;
    Ok(true)
}

pub(crate) async fn remove_trash<F>(
    fs: &Arc<F>,
    path: &Path,
    paths: &OllamaPaths,
    models: Option<&CanonicalDirectory>,
) -> Result<(), OllamaErrorCode>
where
    F: OllamaDurableFs + 'static,
{
    let models = models.ok_or(OllamaErrorCode::OllamaRecoveryDeferred)?;
    let trash = validate_trash(
        path,
        paths
            .active
            .parent()
            .ok_or(OllamaErrorCode::OllamaInternal)?,
        models,
    )?;
    let fs = Arc::clone(fs);
    run_ollama_blocking(move || {
        fs.remove_tree_verified(&trash)
            .map_err(|_| OllamaErrorCode::OllamaStorageUnavailable)
    })
    .await
}

fn present(evidence: &DirectoryEvidence) -> bool {
    matches!(evidence, DirectoryEvidence::Present(_))
}
