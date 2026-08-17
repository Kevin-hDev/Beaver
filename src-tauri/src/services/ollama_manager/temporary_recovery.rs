use super::blocking::run_ollama_blocking;
use super::constants::MAX_DURABLE_DOCUMENT_BYTES;
use super::durable_fs::OllamaDurableFs;
use super::error::OllamaErrorCode;
use super::journal_store::OllamaJournalStore;
use crate::services::paths::{bundle_receipt_path, bundle_receipt_tmp_path, OllamaPaths};
use std::path::Path;
use std::sync::Arc;

pub(super) async fn remove_one<F>(
    fs: &Arc<F>,
    journal: &OllamaJournalStore<F>,
    paths: &OllamaPaths,
) -> Result<bool, OllamaErrorCode>
where
    F: OllamaDurableFs + 'static,
{
    // One durable removal per pass keeps recovery observable and makes every interruption resumable.
    if remove_safe_bundle_receipt_tmp(fs, paths).await? {
        return Ok(true);
    }
    if remove_safe_migration_marker_tmp(fs, paths).await? {
        return Ok(true);
    }
    remove_safe_journal_tmp(fs, journal, paths).await
}

pub(super) async fn remove_safe_bundle_receipt_tmp<F>(
    fs: &Arc<F>,
    paths: &OllamaPaths,
) -> Result<bool, OllamaErrorCode>
where
    F: OllamaDurableFs + 'static,
{
    let temporary = bundle_receipt_tmp_path(&paths.active);
    if !regular_bounded(&temporary, MAX_DURABLE_DOCUMENT_BYTES)? {
        return Ok(false);
    }
    super::bundle_receipt::read_receipt(&**fs, &bundle_receipt_path(&paths.active))?;
    remove(fs, temporary, "bundle-receipt-tmp-remove").await
}

pub(super) async fn remove_safe_migration_marker_tmp<F>(
    fs: &Arc<F>,
    paths: &OllamaPaths,
) -> Result<bool, OllamaErrorCode>
where
    F: OllamaDurableFs + 'static,
{
    if !regular_bounded(&paths.migration_marker_tmp, MAX_DURABLE_DOCUMENT_BYTES)? {
        return Ok(false);
    }
    remove(
        fs,
        paths.migration_marker_tmp.clone(),
        "migration-marker-tmp-remove",
    )
    .await
}

pub(super) async fn remove_safe_journal_tmp<F>(
    fs: &Arc<F>,
    journal: &OllamaJournalStore<F>,
    paths: &OllamaPaths,
) -> Result<bool, OllamaErrorCode>
where
    F: OllamaDurableFs + 'static,
{
    if !regular_bounded(&paths.journal_tmp, MAX_DURABLE_DOCUMENT_BYTES)? {
        return Ok(false);
    }
    journal
        .read()
        .await
        .map_err(|_| OllamaErrorCode::OllamaUpdateRecoveryRequired)?;
    remove(fs, paths.journal_tmp.clone(), "journal-tmp-remove").await
}

fn regular_bounded(path: &Path, limit: usize) -> Result<bool, OllamaErrorCode> {
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => {
            return Err(super::storage_error::io(
                "temporary-document-inspect",
                &error,
                OllamaErrorCode::OllamaUpdateRecoveryRequired,
            ))
        }
    };
    if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() > limit as u64 {
        return Err(OllamaErrorCode::OllamaUpdateRecoveryRequired);
    }
    Ok(true)
}

async fn remove<F>(
    fs: &Arc<F>,
    path: impl Into<std::path::PathBuf>,
    context: &'static str,
) -> Result<bool, OllamaErrorCode>
where
    F: OllamaDurableFs + 'static,
{
    let fs = Arc::clone(fs);
    let path = path.into();
    run_ollama_blocking(move || {
        fs.remove_file_durable(&path)
            .map_err(|error| super::storage_error::durable(context, error))
    })
    .await?;
    Ok(true)
}
