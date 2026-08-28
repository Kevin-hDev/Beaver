use super::cleanup_inspection::{directory_presence, validate_trash};
use super::durable_fs::{platform_fs, OllamaDurableFs};
use super::error::OllamaErrorCode;
use super::journal_store::OllamaJournalStore;
use super::path_identity::CanonicalDirectory;
use super::recovery_decision::ArchiveDirectoryEvidence;
use crate::services::paths::OllamaPaths;
use std::path::Path;
use std::sync::Arc;

pub(super) async fn cleanup(paths: OllamaPaths) -> Result<(), OllamaErrorCode> {
    let models = super::recovery_entry::frozen_models_directory(&paths)
        .ok_or(OllamaErrorCode::OllamaRecoveryDeferred)?;
    cleanup_with(Arc::new(platform_fs()), paths, models).await
}

pub(super) async fn cleanup_with<F>(
    fs: Arc<F>,
    paths: OllamaPaths,
    models: CanonicalDirectory,
) -> Result<(), OllamaErrorCode>
where
    F: OllamaDurableFs + 'static,
{
    let journal = OllamaJournalStore::new(Arc::clone(&fs), paths.clone());
    if journal.read().await?.is_some() || !document_is_absent(&paths.journal_tmp)? {
        return Err(OllamaErrorCode::OllamaUpdateRecoveryRequired);
    }

    // Ces renommages sont les points durables déjà compris par la récupération
    // au démarrage : une interruption reste reprenable sans toucher au bundle actif.
    move_if_present(
        &fs,
        &paths,
        &models,
        &paths.update_staging,
        &paths.uncommitted_staging_delete,
    )
    .await?;
    move_if_present(
        &fs,
        &paths,
        &models,
        &paths.archive_staging,
        &paths.archive_failed,
    )
    .await?;
    remove_if_present(&fs, &paths, &models, &paths.uncommitted_staging_delete).await?;
    remove_if_present(&fs, &paths, &models, &paths.archive_failed).await
}

fn document_is_absent(path: &Path) -> Result<bool, OllamaErrorCode> {
    match std::fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(true),
        Ok(_) => Ok(false),
        Err(_) => Err(OllamaErrorCode::OllamaStorageUnavailable),
    }
}

async fn move_if_present<F>(
    fs: &Arc<F>,
    paths: &OllamaPaths,
    models: &CanonicalDirectory,
    source: &Path,
    destination: &Path,
) -> Result<(), OllamaErrorCode>
where
    F: OllamaDurableFs + 'static,
{
    match directory_presence(source) {
        ArchiveDirectoryEvidence::Absent => return Ok(()),
        ArchiveDirectoryEvidence::Present => {}
        ArchiveDirectoryEvidence::Invalid => {
            return Err(OllamaErrorCode::OllamaUpdateRecoveryRequired)
        }
        ArchiveDirectoryEvidence::Unknown => return Err(OllamaErrorCode::OllamaRecoveryDeferred),
    }
    if !matches!(
        directory_presence(destination),
        ArchiveDirectoryEvidence::Absent
    ) {
        return Err(OllamaErrorCode::OllamaUpdateRecoveryRequired);
    }
    let data_root = paths
        .active
        .parent()
        .ok_or(OllamaErrorCode::OllamaInternal)?;
    validate_trash(source, data_root, models)?;
    super::cleanup::rename(fs, source, destination).await
}

async fn remove_if_present<F>(
    fs: &Arc<F>,
    paths: &OllamaPaths,
    models: &CanonicalDirectory,
    path: &Path,
) -> Result<(), OllamaErrorCode>
where
    F: OllamaDurableFs + 'static,
{
    match directory_presence(path) {
        ArchiveDirectoryEvidence::Absent => Ok(()),
        ArchiveDirectoryEvidence::Present => {
            super::cleanup::remove_trash(fs, path, paths, Some(models)).await
        }
        ArchiveDirectoryEvidence::Invalid => Err(OllamaErrorCode::OllamaUpdateRecoveryRequired),
        ArchiveDirectoryEvidence::Unknown => Err(OllamaErrorCode::OllamaRecoveryDeferred),
    }
}
