#![allow(dead_code)]

use super::cleanup;
use super::cleanup_inspection::validate_trash;
use super::durable_fs::OllamaDurableFs;
use super::error::OllamaErrorCode;
use super::path_identity::CanonicalDirectory;
use super::recovery_decision::{ArchiveDirectoryEvidence, OllamaLayoutSnapshot};
use crate::services::paths::OllamaPaths;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ArchiveRecoveryAction {
    MoveToFailed,
    RemoveFailed,
}

pub(crate) fn decide(
    snapshot: &OllamaLayoutSnapshot,
) -> Result<Option<ArchiveRecoveryAction>, OllamaErrorCode> {
    if [snapshot.archive_staging, snapshot.archive_failed]
        .into_iter()
        .any(|evidence| {
            matches!(
                evidence,
                ArchiveDirectoryEvidence::Unknown | ArchiveDirectoryEvidence::Invalid
            )
        })
    {
        return Err(OllamaErrorCode::OllamaRecoveryDeferred);
    }
    match (snapshot.archive_staging, snapshot.archive_failed) {
        (ArchiveDirectoryEvidence::Absent, ArchiveDirectoryEvidence::Absent) => Ok(None),
        (ArchiveDirectoryEvidence::Present, ArchiveDirectoryEvidence::Absent) => {
            Ok(Some(ArchiveRecoveryAction::MoveToFailed))
        }
        (ArchiveDirectoryEvidence::Absent, ArchiveDirectoryEvidence::Present) => {
            Ok(Some(ArchiveRecoveryAction::RemoveFailed))
        }
        (ArchiveDirectoryEvidence::Present, ArchiveDirectoryEvidence::Present) => {
            Err(OllamaErrorCode::OllamaRecoveryDeferred)
        }
        _ => Err(OllamaErrorCode::OllamaRecoveryDeferred),
    }
}

pub(crate) async fn apply<F>(
    action: ArchiveRecoveryAction,
    fs: &Arc<F>,
    paths: &OllamaPaths,
    models: Option<&CanonicalDirectory>,
) -> Result<(), OllamaErrorCode>
where
    F: OllamaDurableFs + 'static,
{
    let models = models.ok_or(OllamaErrorCode::OllamaRecoveryDeferred)?;
    match action {
        ArchiveRecoveryAction::MoveToFailed => {
            validate_archive(&paths.archive_staging, paths, models)?;
            cleanup::rename(fs, &paths.archive_staging, &paths.archive_failed).await
        }
        ArchiveRecoveryAction::RemoveFailed => {
            cleanup::remove_trash(fs, &paths.archive_failed, paths, Some(models)).await
        }
    }
}

fn validate_archive(
    path: &std::path::Path,
    paths: &OllamaPaths,
    models: &CanonicalDirectory,
) -> Result<CanonicalDirectory, OllamaErrorCode> {
    let data_root = paths
        .active
        .parent()
        .ok_or(OllamaErrorCode::OllamaInternal)?;
    validate_trash(path, data_root, models)
}
