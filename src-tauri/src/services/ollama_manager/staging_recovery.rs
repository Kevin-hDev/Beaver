use super::cleanup;
use super::cleanup_inspection::{directory_presence, validate_trash};
use super::durable_fs::OllamaDurableFs;
use super::error::OllamaErrorCode;
use super::path_identity::CanonicalDirectory;
use super::recovery_decision::{ArchiveDirectoryEvidence, JournalPresence};
use crate::services::paths::OllamaPaths;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum StagingSource {
    Install,
    Update,
    Legacy,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum StagingRecoveryAction {
    Move(StagingSource),
    RemoveTrash,
}

pub(super) fn decide(
    journal: &JournalPresence,
    paths: &OllamaPaths,
) -> Result<Option<StagingRecoveryAction>, OllamaErrorCode> {
    let sources = [
        (
            StagingSource::Install,
            directory_presence(&paths.install_staging),
        ),
        (
            StagingSource::Update,
            directory_presence(&paths.update_staging),
        ),
        (
            StagingSource::Legacy,
            directory_presence(&paths.legacy_staging),
        ),
    ];
    let trash = directory_presence(&paths.uncommitted_staging_delete);
    let evidence = sources
        .iter()
        .map(|(_, evidence)| *evidence)
        .chain(std::iter::once(trash));
    if evidence
        .clone()
        .any(|value| matches!(value, ArchiveDirectoryEvidence::Invalid))
    {
        return Err(OllamaErrorCode::OllamaUpdateRecoveryRequired);
    }
    if evidence
        .clone()
        .any(|value| matches!(value, ArchiveDirectoryEvidence::Unknown))
    {
        return Err(OllamaErrorCode::OllamaRecoveryDeferred);
    }
    let present = sources
        .iter()
        .filter_map(|(source, evidence)| {
            matches!(evidence, ArchiveDirectoryEvidence::Present).then_some(*source)
        })
        .collect::<Vec<_>>();
    if !matches!(journal, JournalPresence::Absent) {
        return if matches!(trash, ArchiveDirectoryEvidence::Absent) {
            Ok(None)
        } else {
            Err(OllamaErrorCode::OllamaUpdateRecoveryRequired)
        };
    }
    match (present.as_slice(), trash) {
        ([], ArchiveDirectoryEvidence::Absent) => Ok(None),
        ([], ArchiveDirectoryEvidence::Present) => Ok(Some(StagingRecoveryAction::RemoveTrash)),
        ([source], ArchiveDirectoryEvidence::Absent) => {
            Ok(Some(StagingRecoveryAction::Move(*source)))
        }
        _ => Err(OllamaErrorCode::OllamaUpdateRecoveryRequired),
    }
}

pub(super) async fn apply<F>(
    action: StagingRecoveryAction,
    fs: &Arc<F>,
    paths: &OllamaPaths,
    models: Option<&CanonicalDirectory>,
) -> Result<(), OllamaErrorCode>
where
    F: OllamaDurableFs + 'static,
{
    let models = models.ok_or(OllamaErrorCode::OllamaRecoveryDeferred)?;
    match action {
        StagingRecoveryAction::Move(source) => {
            let source = match source {
                StagingSource::Install => &paths.install_staging,
                StagingSource::Update => &paths.update_staging,
                StagingSource::Legacy => &paths.legacy_staging,
            };
            let data_root = paths
                .active
                .parent()
                .ok_or(OllamaErrorCode::OllamaInternal)?;
            validate_trash(source, data_root, models)?;
            cleanup::rename(fs, source, &paths.uncommitted_staging_delete).await
        }
        StagingRecoveryAction::RemoveTrash => {
            cleanup::remove_trash(fs, &paths.uncommitted_staging_delete, paths, Some(models)).await
        }
    }
}
